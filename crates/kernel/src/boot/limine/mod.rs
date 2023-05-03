//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::limine_reqs;
use nd_limine::{
    BootloaderInfo, EntryPoint, File, KernelAddress, MemMapEntryType, MemoryMap, Module, Request,
};

use crate::arch::x86_64::MemorySegment;

/// Requests the bootloader to provide information about itself, such as its name and version.
/// Those information will be logged at startup.
static BOOTLOADER_INFO: Request<BootloaderInfo> = Request::new(BootloaderInfo);

/// Requests the Limine bootloader to call a specific function rather than the entry point specified
/// in the ELF header.
static ENTRY_POINT: Request<EntryPoint> = Request::new(EntryPoint(entry_point));

/// Requests Limine to load an additional module along with the kernel itself.
///
/// This module will contain the initial program to start after the kernel has initialize itself.
static MODULE: Request<Module> = Request::new(Module::new(&[]));

/// Requests the Limine bootloader to provide the address of the kernel in physical memory.
static KERNEL_ADDRESS: Request<KernelAddress> = Request::new(KernelAddress);

/// Requests the Limine bootloader to provide a map of the available physical memory.
static MEMORY_MAP: Request<MemoryMap> = Request::new(MemoryMap);

limine_reqs!(
    MEMORY_MAP,
    BOOTLOADER_INFO,
    MODULE,
    ENTRY_POINT,
    KERNEL_ADDRESS,
);

/// Removes the begining of a path, only keeping the what's after the last `/` character.
fn get_filename(bytes: &[u8]) -> &[u8] {
    let start_idx = match bytes.iter().rposition(|&b| b == b'/') {
        Some(slash) => slash + 1,
        None => 0,
    };

    unsafe { bytes.get_unchecked(start_idx..) }
}

/// Reads The content of the [`MODULE`] request and returns the file that has been loaded.
///
/// # Panics
///
/// If the init program is not present, this function panics with an appropriate error message.
fn find_init_program() -> Option<&'static File> {
    nd_log::trace!("Enumerating kernel modules...");

    let response = MODULE.response()?;

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

/// The entry point of the kernel when booted by the Limine bootloader.
extern "C" fn entry_point() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        crate::arch::x86_64::initialize_logger();
        #[cfg(target_arch = "x86_64")]
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

    let Some(kernel_address) = KERNEL_ADDRESS.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::arch::die();
    };

    nd_log::trace!(
        "Kernel located at {:#x}, mapped at {:#x}.",
        kernel_address.physical_base(),
        kernel_address.virtual_base()
    );

    // Bootloader reclaimable memory and useable memory segments can be used by the kernel.
    let mut available_memory = memmap
        .entries()
        .iter()
        .filter(|e| matches!(e.ty(), MemMapEntryType::USABLE))
        .map(|e| MemorySegment {
            base: e.base(),
            length: e.length(),
        });

    // SAFETY:
    //  We're in the entry point, this function won't ever be called again.
    //  The Limine bootloader identity maps the whole address space, from 0x1000 up to roughly
    //  four gigabytes, ensuring that the page tables are properly identity mapped.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        crate::arch::x86_64::initialize_paging(&mut available_memory);
    }

    // Note:
    //  From this point, accessing any memory owned by the bootloader is undefined behavior.
    //  The `initialize_paging` function may have overwritten the bootloader's memory.

    crate::init::load(nd_init.data());
}
