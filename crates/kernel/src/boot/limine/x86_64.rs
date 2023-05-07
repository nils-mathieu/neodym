//! Contains the Limine entry point on **x86_64**.
//!
//! See [`entry_point`].

use nd_limine::MemMapEntryType;
use nd_x86_64::VirtAddr;

use crate::arch::x86_64::{
    KernelInfo, KernelInfoTok, MemoryMapper, MemorySegment, OutOfPhysicalMemory, PageAllocatorTok,
    Process,
};

use super::find_init_program;
use super::{BOOTLOADER_INFO, HHDM, KERNEL_ADDR, MEMORY_MAP};

/// The entry point of the kernel when booted by the Limine bootloader on **x86_64**.
pub extern "C" fn entry_point() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe {
        crate::arch::x86_64::initialize_logger();
        crate::arch::x86_64::initialize_tables();
    }

    if let Some(info) = BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})!", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader.");
    }

    let Some(memmap) = MEMORY_MAP.response() else {
        nd_log::error!("The Limine bootloader did not provide a map of the usable memory.");
        crate::arch::die();
    };

    let Some(kernel_addr) = KERNEL_ADDR.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::arch::die();
    };

    let Some(hhdm) = HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM offset.");
        crate::arch::die();
    };

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

    // Bootloader reclaimable memory and useable memory segments can be used by the kernel.
    let mut available_memory = memmap
        .entries()
        .iter()
        .filter(|e| matches!(e.ty(), MemMapEntryType::USABLE))
        .map(|e| MemorySegment {
            base: e.base(),
            length: e.length(),
        });

    for entry in memmap.entries() {
        nd_log::trace!(
            "Memory segment: {:#x} - {:#x} ({:?})",
            entry.base(),
            entry.base() + entry.length(),
            entry.ty()
        );
    }

    let page_allocator = unsafe {
        // Initialize some global systems required by the kernel's system and some interrupt
        // handlers, such as the page allocator, local APICs, etc.
        //
        // When we have support for multiple CPUs, initialization code will come here.
        let info = KernelInfoTok::initialize(KernelInfo {
            kernel_addr: kernel_addr.physical_base(),
            kernel_size: 0,
            hhdm_offset: hhdm.offset(),
        });

        let page_allocator = PageAllocatorTok::initialize(info, &mut available_memory);
        crate::arch::x86_64::initialize_lapic();

        // Enable interrupts. We're ready to be interrupted x).
        nd_x86_64::sti();

        page_allocator
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

    let mut process = Process {
        instruction_pointer: LOADED_AT,
        memory_mapper: MemoryMapper::new(page_allocator)?,
    };

    Ok(())
}
