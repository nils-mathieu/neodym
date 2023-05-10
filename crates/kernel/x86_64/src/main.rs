//! # The Neodym Operating System
//!
//! The present documentation describes the internal architecture of Neodym, including some
//! implementation details.
//!
//! Because of the architecture-specific nature of the kernel, this documentation is only
//! relevent for the `x86_64` architecture.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(used_with_arg)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(asm_const)]

/// Returns the size of the kernel image, in bytes.
fn image_size() -> usize {
    // This is a symbol defined in the linker script. Its *address* will be defined to the size of
    // unpacked kernel image.
    extern "C" {
        #[link_name = "__nd_image_size"]
        static IMAGE_SIZE: u8;
    }

    // SAFETY:
    //  This static external variable is set by the linker script, and won't change afterwards.
    unsafe { &IMAGE_SIZE as *const u8 as usize }
}

mod boot;

mod apic;
mod interrupts;
mod logger;
mod sys_info;
mod tables;

/// Disables interrupts and halts the CPU.
///
/// This function can be called when an unrecoverable error occurs.
fn die() -> ! {
    unsafe {
        nd_x86_64::cli();

        loop {
            nd_x86_64::hlt();
        }
    }
}

/// This function is called when something in our code panics. This should be considered a serious
/// bug in the kernel.
#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    nd_log::error!("KERNEL PANIC!");
    nd_log::error!("");
    nd_log::error!("  This is a serious bug in the kernel.");
    nd_log::error!("  Please report this issue at");
    nd_log::error!("");
    nd_log::error!("      https://github.com/nils-mathieu/neodym/issues/new");
    nd_log::error!("");

    if let Some(message) = info.message() {
        nd_log::error!("> Message: {}", message);
    }

    if let Some(location) = info.location() {
        nd_log::error!(">      At: {}:{}", location.file(), location.line());
    }

    die();
}
