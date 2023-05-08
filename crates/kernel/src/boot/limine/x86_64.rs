//! Contains the Limine entry point on **x86_64**.
//!
//! See [`entry_point`].

use nd_limine::MemMapEntryType;
use nd_x86_64::{PageTableFlags, VirtAddr};

use crate::arch::x86_64::{
    KernelInfo, KernelInfoTok, MappingError, MemoryMapper, MemorySegment, OutOfPhysicalMemory,
    PageAllocatorTok, ProcessInfo,
};

use super::find_init_program;
use super::{BOOTLOADER_INFO, HHDM, KERNEL_ADDR, MEMORY_MAP};

static mut KERNEL_STACK: [u8; 4096 * 16] = [0; 4096 * 16];

/// The entry point of the kernel when booted by the Limine bootloader on **x86_64**.
pub extern "C" fn entry_point() -> ! {
    // Setup the stack used by the kernel.
    unsafe {
        let stack = KERNEL_STACK.as_mut_ptr().add(4096 * 16) as usize as VirtAddr;
        nd_x86_64::set_rsp(stack);
    }

    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe { crate::arch::x86_64::initialize_logger() };

    if let Some(info) = BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})!", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader.");
    }

    let Some(kernel_addr) = KERNEL_ADDR.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::arch::die();
    };

    let Some(hhdm) = HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM offset.");
        crate::arch::die();
    };

    // Initialize some global systems required by the kernel's system and some interrupt
    // handlers, such as the page allocator, local APICs, etc.
    //
    // When we have support for multiple CPUs, initialization code will come here.
    let kernel_info = unsafe {
        KernelInfoTok::initialize(KernelInfo {
            kernel_phys_addr: kernel_addr.physical_base(),
            kernel_virt_addr: kernel_addr.virtual_base(),
            kernel_size: crate::image_size(),
            hhdm_offset: hhdm.offset(),
        })
    };

    let Some(memmap) = MEMORY_MAP.response() else {
        nd_log::error!("The Limine bootloader did not provide a map of the usable memory.");
        crate::arch::die();
    };

    // Bootloader reclaimable memory and useable memory segments can be used by the kernel.
    let mut available_memory = memmap
        .entries()
        .iter()
        .filter(|e| matches!(e.ty(), MemMapEntryType::USABLE))
        .map(|e| MemorySegment {
            base: e.base(),
            length: e.length(),
        });

    let page_allocator =
        unsafe { PageAllocatorTok::initialize(kernel_info, &mut available_memory) };

    unsafe {
        crate::arch::x86_64::setup_gdt();
        crate::arch::x86_64::setup_idt();
        crate::arch::x86_64::setup_system_calls();
    }

    // Load the initial program.
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
        crate::arch::die();
    };

    unsafe {
        crate::arch::x86_64::initialize_lapic();

        // Enable interrupts. We're ready to be interrupted x).
        nd_x86_64::sti();
    };

    match spawn_nd_init(page_allocator, nd_init.data()) {
        Ok(()) => (),
        Err(OutOfPhysicalMemory) => {
            nd_log::error!("Not enough memory to spawn the initial program.");
            crate::arch::die();
        }
    };

    todo!("Start the scheduler here.");
}

fn spawn_nd_init(page_allocator: PageAllocatorTok, data: &[u8]) -> Result<(), OutOfPhysicalMemory> {
    // Start the initial program! This is the end of the boot process.
    const LOADED_AT: VirtAddr = 0x10_0000;
    const STACK_SIZE_IN_PAGES: u64 = 16;
    const STACK_BASE: VirtAddr = LOADED_AT - 0x1000;

    nd_log::trace!("Loading the `nd_init` program...");

    let mut process = ProcessInfo {
        instruction_pointer: LOADED_AT,
        memory_mapper: MemoryMapper::new(page_allocator)?,
        stack_pointer: STACK_BASE,
    };

    // Map the process itself into its address space.
    match process.memory_mapper.load_at(
        data,
        LOADED_AT,
        PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
    ) {
        Ok(()) => (),
        Err(MappingError::OutOfPhysicalMemory) => return Err(OutOfPhysicalMemory),
        Err(MappingError::AlreadyMapped(_)) => {
            debug_assert!(false, "nothing should be mapped here...");
            unsafe { core::hint::unreachable_unchecked() };
        }
    }

    match process.memory_mapper.load_at_with(
        STACK_BASE - STACK_SIZE_IN_PAGES * 0x1000,
        STACK_SIZE_IN_PAGES,
        PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
        |_| (),
    ) {
        Ok(_) => (),
        Err(MappingError::OutOfPhysicalMemory) => return Err(OutOfPhysicalMemory),
        Err(MappingError::AlreadyMapped(_)) => {
            debug_assert!(false, "nothing should be mapped here...");
            unsafe { core::hint::unreachable_unchecked() };
        }
    }

    nd_log::info!("Passing control to the `nd_init` program...");

    crate::arch::x86_64::spawn(process)?;

    Ok(())
}
