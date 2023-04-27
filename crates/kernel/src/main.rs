//! # The Neodym Operating System
//!
//! The present documentation describes the internal architecture of Neodym, including some
//! implementation details.
//!
//! ## Boot Sequence
//!
//! This part of the kernel expects the machine to be loaded in a specific machine state detailed
//! in the different sub-modules of the [`arch`] module (see [`arch::entry_point`]). However, the
//! literal entry points of the kernel are defined under the [`boot`] module.
//!
//! Those two modules are different because some bootloaders (such as
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md)) may support
//! multiple CPU architectures.

#![no_std]
#![no_main]

mod arch;
mod boot;

/// This function is called when something in our code panics. This should be considered a serious
/// bug in the kernel.
#[panic_handler]
fn handle_panic(_info: &core::panic::PanicInfo) -> ! {
    // TODO: Log the error properly.
    self::arch::die();
}
