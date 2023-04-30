//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::{limine_reqs, BootloaderInfoRequest};
use nd_limine::{EntryPointRequest, Request};

/// Requests the bootloader to provide information about itself, such as its name and version.
/// Those information will be logged at startup.
static BOOTLOADER_INFO: Request<BootloaderInfoRequest> = Request::new(BootloaderInfoRequest);

/// Requests the Limine bootloader to call a specific function rather than the entry point specified
/// in the ELF header.
static ENTRY_POINT: Request<EntryPointRequest> = Request::new(EntryPointRequest(entry_point));

limine_reqs!(BOOTLOADER_INFO, ENTRY_POINT);

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

    todo!("reached the end of `crate::boot::limine::entry_point`");
}
