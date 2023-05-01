//! This program is the first thing that will be loaded by the kernel after it has been initailized.
//!
//! It is responsible for initializing the user's environment.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    // TODO:
    //  Exit the process properly.
    loop {}
}

/// The entry point of the program.
///
/// This function called by the kernel once it has properly initialized itself.
#[no_mangle]
extern "C" fn entry_point() -> ! {
    todo!();
}
