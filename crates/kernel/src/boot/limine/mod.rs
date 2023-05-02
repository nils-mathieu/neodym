//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::limine_reqs;
use nd_limine::{BootloaderInfo, EntryPoint, File, Module, Request, Smp, SmpRequestFlags};

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

/// Requests the Limine bootloader to gather and provide information about the other processors,
/// such as their local APIC ID.
static SMP: Request<Smp> = Request::new(Smp {
    flags: SmpRequestFlags::X2APIC,
});

limine_reqs!(SMP, BOOTLOADER_INFO, MODULE, ENTRY_POINT);

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
    unsafe {
        crate::arch::initialize();
    }

    if let Some(info) = BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader");
    }

    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    let Some(_smp) = SMP.response() else {
        nd_log::error!("The Limine bootloader did not provide any information about other CPUs");
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

    crate::init::load(nd_init.data());
}
