//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::limine_reqs;
use nd_limine::{
    BootloaderInfo, EntryPoint, File, Hhdm, KernelAddress, MemoryMap, Module, Request,
};

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
use self::x86_64::entry_point;

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

/// Requests the Limine bootloader to provide a map of the available physical memory.
static MEMORY_MAP: Request<MemoryMap> = Request::new(MemoryMap);

/// Requests the Limine bootloader to provide the address of the kernel in physical memory.
static KERNEL_ADDR: Request<KernelAddress> = Request::new(KernelAddress);

/// The Limine bootloader maps the entierty of the physical memory to the higher half of the
/// virtual address space.
///
/// This request provides the address of th *Higher Half Direct Map* offset.
static HHDM: Request<Hhdm> = Request::new(Hhdm);

limine_reqs!(
    MEMORY_MAP,
    BOOTLOADER_INFO,
    MODULE,
    ENTRY_POINT,
    HHDM,
    KERNEL_ADDR
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
