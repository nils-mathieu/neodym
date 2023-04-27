//! # The Neodym Operating System
//!
//! The present documentation describes the internal architecture of Neodym, including some
//! implementation details.

#![no_std]
#![no_main]

/// This function is called when something in our code panics. This should be considered a serious
/// bug in the kernel.
#[panic_handler]
fn handle_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
