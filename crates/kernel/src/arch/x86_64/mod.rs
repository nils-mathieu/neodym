//! Code specific to the `x86_64` CPU architecture.

/// Disables interrupts and halts the CPU.
///
/// This function can be called when an unrecoverable error occurs.
pub fn die() -> ! {
    use core::arch::asm;

    unsafe {
        asm!("cli; hlt", options(noreturn));
    }
}

/// The entry point of the kernel on the `x86_64` architecture.
///
/// It should normally be called at the end of bootloader-specific entry points.
///
/// # Expected Machine State
///
/// # Safety
///
/// The CPU must be in the *expected machine state*.
///
/// This function must only be called once (e.g. it must not be called from within itself).
pub unsafe fn entry_point() -> ! {
    die();
}
