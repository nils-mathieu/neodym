//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::limine_reqs;
use nd_limine::{BootloaderInfo, EntryPoint, File, Module, Request};

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

limine_reqs!(BOOTLOADER_INFO, MODULE, ENTRY_POINT);

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
fn load_init_program() -> Option<&'static File> {
    nd_log::trace!("Loading kernel modules...");

    let response = MODULE.response()?;

    let mut found = None;

    for module in response.modules().iter().filter_map(|x| x.file()) {
        nd_log::trace!(" - {:?} ({:?})", module.path(), module.cmdline());

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
        crate::arch::initialize();
    }

    if let Some(info) = BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader");
    }

    // Load the initial program.
    let Some(nd_init) = load_init_program() else {
        nd_log::error!("An `nd_init` module is expected along with the kernel.");  
        nd_log::error!("Check your Limine config!");
        nd_log::error!("");
        nd_log::error!("Example `limine.cfg`:");
        nd_log::error!("");
        nd_log::error!("    PROTOCOL=limine");
        nd_log::error!("    KERNEL_PATH=boot:///neodym");
        nd_log::error!("    MODULE_PATH=boot:///nd_init");
        nd_log::error!("    MODULE_CMDLINE=elf");
                    nd_log::error!("");
        crate::arch::die();
    };

    let nd_init_ty = if nd_init.cmdline().to_bytes() == b"" {
        crate::init::FileType::Guess
    } else {
        match crate::init::FileType::from_bytes(nd_init.cmdline().to_bytes()) {
            Some(ty) => ty,
            None => {
                nd_log::error!("The file type {:?} is not valid.", nd_init.cmdline());
                nd_log::error!("Supported file types are:");
                nd_log::error!("  - guess");
                nd_log::error!("  - elf");
                nd_log::error!("  - bin");
                crate::arch::die();
            }
        }
    };

    let entry_point = match crate::init::parse_entry_point(nd_init_ty, nd_init.data()) {
        Ok(offset) => offset,
        Err(err) => {
            use crate::init::EntryPointError;

            match err {
                EntryPointError::CantGuess => {
                    nd_log::error!("The type of the `nd_init` module cannot be guessed.");
                    nd_log::error!("Please provide a type using the CMDLINE argument.");
                    nd_log::error!("");
                    nd_log::error!("Example `limine.cfg`:");
                    nd_log::error!("");
                    nd_log::error!("    PROTOCOL=limine");
                    nd_log::error!("    KERNEL_PATH=boot:///neodym");
                    nd_log::error!("    MODULE_PATH=boot:///nd_init");
                    nd_log::error!("    MODULE_CMDLINE=elf");
                    nd_log::error!("");
                }
                EntryPointError::InvalidElfHeader => {
                    nd_log::error!("The ELF header of `nd_init` is invalid.");
                }
            }
            crate::arch::die();
        }
    };

    nd_log::trace!("Entry Point Offset: {:x}", entry_point);

    todo!("reached the end of `crate::boot::limine::entry_point`");
}
