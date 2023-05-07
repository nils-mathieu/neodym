//! This program is the first thing that will be loaded by the kernel after it has been initailized.
//!
//! It is responsible for initializing the user's environment.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    neodym_sys::terminate_self();
}

/// The entry point of the program.
///
/// This function called by the kernel once it has properly initialized itself.
#[link_section = ".entry_point"]
#[no_mangle]
extern "C" fn entry_point() -> ! {
    main();
    neodym_sys::terminate_self();
}

/// The main function of the program.
///
/// This function is called by the raw [`entry_point`] upon startup of the program
/// and is responsible for initializing the user's environment.
fn main() {
    // Initialize a simple text-mode environment.
}
