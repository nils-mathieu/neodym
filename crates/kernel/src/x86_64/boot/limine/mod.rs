//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.
//!

use nd_limine::{File, MemMapEntryType};
use nd_x86_64::{Cr3, Cr3Flags, PageTable, PageTableFlags, VirtAddr};

use crate::x86_64::mapping::MappingError;
use crate::x86_64::{
    MemorySegment, OwnedMapper, PageAllocatorTok, PageProvider, SysInfo, SysInfoTok,
};

mod req;

/// Removes the begining of a path, only keeping the what's after the last `/` character.
fn get_filename(bytes: &[u8]) -> &[u8] {
    let start_idx = match bytes.iter().rposition(|&b| b == b'/') {
        Some(slash) => slash + 1,
        None => 0,
    };

    unsafe { bytes.get_unchecked(start_idx..) }
}

/// Reads The content of the "MODULE" request and returns the file that has been loaded.
///
/// # Panics
///
/// If the init program is not present, this function panics with an appropriate error message.
fn find_init_program() -> Option<&'static File> {
    nd_log::trace!("Enumerating kernel modules...");

    let response = req::MODULE.response()?;

    let mut found = None;

    for module in response.modules().iter().filter_map(|x| x.file()) {
        nd_log::trace!(" - {:?}", module.path());

        // We're looking for a file named 'nd_init'.
        if get_filename(module.path().to_bytes()) == b"nd_init" {
            found = Some(module);
        }
    }

    found
}

const KERNEL_STACK_SIZE: usize = 4096 * 16;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

/// The entry point of the kernel when booted by the Limine bootloader on **x86_64**.
#[naked]
extern "C" fn entry_point() -> ! {
    unsafe {
        // We need to setup a stack within the kernel's address space, as the bootloader's memory
        // eventually gets reclaimed as usable memory.
        //
        // NOTE:
        //  We can use JMP instead of a regular CALL as the function won't return anyway.
        core::arch::asm!(
            r#"
            lea rsp, [{} + {}]
            mov rbp, rsp
            jmp {}
            "#,
            sym KERNEL_STACK,
            const KERNEL_STACK_SIZE,
            sym entry_point_inner,
            options(noreturn),
        );
    }
}

extern "C" fn entry_point_inner() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe { crate::x86_64::initialize_logger() };

    //
    // Gather the responses from the Limine bootloader.
    // Some are necessary, others are just nice information to have.
    //

    if let Some(info) = req::BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})!", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader.");
    }

    if req::ENTRY_POINT.response().is_none() {
        nd_log::warn!("The Limine bootloader did not respond to the entry point request.");
        nd_log::warn!("  > This is just a sanity check.");
        nd_log::warn!("  > The bootloader might be corrupted.");
    }

    let Some(kernel_addr) = req::KERNEL_ADDR.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::die();
    };

    let Some(memmap) = req::MEMORY_MAP.response() else {
        nd_log::error!("The Limine bootloader did not provide a map of the usable memory.");
        crate::die();
    };

    let Some(hhdm) = req::HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM address.");
        crate::die();
    };

    let Some(nd_init) = find_init_program() else {
        nd_log::error!("An `nd_init` module is expected along with the kernel.");
        nd_log::error!("Check your Limine config!");
        nd_log::error!("");
        nd_log::error!("Example `limine.cfg`:");
        nd_log::error!("");
        nd_log::error!("    PROTOCOL=limine");
        nd_log::error!("    KERNEL_PATH=boot:///neodym");
        nd_log::error!("    MODULE_PATH=boot:///nd_init");
        nd_log::error!("");
        crate::die();
    };

    let kernel_virt_addr = SysInfo::read_kernel_virt_addr();

    if kernel_virt_addr != kernel_addr.virtual_base() {
        nd_log::error!("The kernel was not loaded at the expected address.");
        nd_log::error!("  > Expected: {:#x}", kernel_virt_addr);
        nd_log::error!("  > Actual:   {:#x}", kernel_addr.virtual_base());
        nd_log::error!("How is this code even running?");
        nd_log::error!("");
        nd_log::error!("This is a bug in your bootloader.");
        crate::die();
    }

    // This iterator goes over every memory segment that is available for the kernel to use.
    let mut available_mem = memmap
        .entries()
        .iter()
        .map(|&&e| e)
        .filter(|e| {
            e.ty() == MemMapEntryType::USABLE || e.ty() == MemMapEntryType::BOOTLOADER_RECLAIMABLE
        })
        .map(|e| MemorySegment {
            base: e.base(),
            length: e.length(),
        });

    let page_provider = PageProvider::new(&mut available_mem);

    let kernel_virt_end_addr = SysInfo::read_kernel_virt_end_addr();

    let physical_memory_size = match memmap
        .entries()
        .iter()
        .filter(|e| e.ty() != MemMapEntryType::RESERVED)
        .last()
    {
        Some(e) => e.base() + e.length(),
        None => 0,
    };

    let kernel_phys_addr = kernel_addr.physical_base();
    let hhdm_start = hhdm.offset();

    // Initialize the global kernel info object.
    //
    // This is used throughout the kernel to access information about the kernel and the system
    // that the kernel is running on.
    let sys_info = unsafe {
        SysInfoTok::initialize(SysInfo {
            kernel_phys_addr,
            kernel_virt_addr,
            kernel_virt_end_addr,
            hhdm_start,
        })
    };

    // Initialize the CPU in a well-known state.
    //
    // SAFETY:
    //  Thos function must only be called once. We're still in the entry point, which is only
    //  called once by the bootloader.
    let pml4 = unsafe {
        crate::x86_64::setup_gdt();
        crate::x86_64::setup_idt();
        crate::x86_64::setup_system_calls();
        crate::x86_64::initialize_lapic();

        match crate::x86_64::mapping::generate_page_table(
            &page_provider,
            &mut |phys| phys + hhdm_start,
            physical_memory_size,
            kernel_phys_addr,
            kernel_virt_addr,
            kernel_virt_end_addr - kernel_virt_addr,
            hhdm_start,
        ) {
            Ok(pml4) => pml4,
            Err(_err) => {
                nd_log::error!("Not enough memory to setup paging.");
                #[cfg(debug_assertions)]
                nd_log::error!("  > Error: {:?}", _err);
                crate::die();
            }
        }
    };

    let page_allocator = unsafe { PageAllocatorTok::initialize(sys_info, page_provider) };

    unsafe {
        nd_log::trace!("Switching up address space...");
        nd_x86_64::set_cr3(Cr3::new(pml4, Cr3Flags::empty()));
    }

    unsafe {
        // Enable interrupts. We're ready to be interrupted x).
        nd_x86_64::sti();
    }

    match spawn_init_process(page_allocator, nd_init.data()) {
        Ok(()) => (),
        Err(MappingError::AlreadyMapped) => {
            debug_assert!(
                false,
                "something is already mapped at the init process address"
            );
            unsafe { core::hint::unreachable_unchecked() };
        }
        Err(MappingError::OutOfPhysicalMemory) => {
            nd_log::error!("Not enough physical memory to load `nd_init`.");
            crate::die();
        }
    }

    todo!();
}

/// Initializes the `nd_init` process.
fn spawn_init_process(
    page_allocator: PageAllocatorTok,
    nd_init: &[u8],
) -> Result<(), MappingError> {
    let mut owned_mapper = OwnedMapper::new(page_allocator)?;

    // Map the kernel and the init process into the address space.
    // We know that those are always present regardless of the current address space, so we can
    // just copy those entries from the current address space.
    let current = unsafe {
        &mut *((nd_x86_64::cr3().addr() + page_allocator.sys_info().hhdm_start) as *mut PageTable)
    };

    for i in 256..512 {
        let entry = unsafe { current.get_unchecked_mut(i) };

        if entry.flags().contains(PageTableFlags::PRESENT) {
            let dst = unsafe { owned_mapper.pml4_mut().get_unchecked_mut(i) };
            *dst = *entry;
        }
    }

    // Map the `nd_init` process at address `0x10_0000`.
    const LOAD_ADDR: VirtAddr = 0x10_0000;
    const STACK_SIZE: u64 = 64 * 1024;
    const STACK_TOP: VirtAddr = LOAD_ADDR - 0x1000;

    owned_mapper.load(
        LOAD_ADDR,
        nd_init,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    )?;

    owned_mapper.load_uninit(
        STACK_TOP - STACK_SIZE,
        STACK_SIZE / 0x1000,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    )?;

    // Create a 64 KiB stack for the process.

    unsafe { owned_mapper.switch() };

    unsafe {
        core::arch::asm!(
            r#"
            mov rcx, {}
            mov rsp, {}
            mov rbp, rsp
            sysretq
            "#,
            const LOAD_ADDR,
            const STACK_TOP,
        );
    }

    Ok(())
}
