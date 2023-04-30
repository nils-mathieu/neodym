//! Code specific to the `x86_64` CPU architecture.

mod interrupts;
mod logger;
mod tables;

/// Disables interrupts and halts the CPU.
///
/// This function can be called when an unrecoverable error occurs.
pub fn die() -> ! {
    unsafe {
        nd_x86_64::cli();

        loop {
            nd_x86_64::hlt();
        }
    }
}

/// Initializes the CPU and puts it into the
///
/// It should normally be called at the end of bootloader-specific entry points.
///
/// # Steps
///
/// This function will initialize the logging facade, setup a **GDT** and an **IDT**.
///
/// # Expected Machine State
///
/// The CPU must be in 64-bit long mode. The IDT may be in an uninitialized state.
///
/// # Safety
///
/// The CPU must be in the *expected machine state*.
///
/// This function must only be called once (e.g. it must not be called from within itself).
pub unsafe fn initialize() {
    unsafe {
        self::logger::initialize();
        self::tables::initialize();
    }
}
