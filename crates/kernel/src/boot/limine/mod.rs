//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.

use nd_limine::limine_reqs;
use nd_limine::{EntryPointRequest, Request};

/// Requests the Limine bootloader to call a specific function rather than the entry point specified
/// in the ELF header.
static ENTRY_POINT: Request<EntryPointRequest> = Request::new(EntryPointRequest(entry_point));

limine_reqs!(ENTRY_POINT);

/// The entry point of the kernel when booted by the Limine bootloader.
extern "C" fn entry_point() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe { crate::arch::entry_point() }
}
