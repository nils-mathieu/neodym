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

/// Initializes the CPU.
///
/// It should normally be called at the begining of bootloader-specific entry points in order to
/// put the machine in an stable state, suitable for actually initializing the kernel interface.
///
/// # Steps
///
/// 1. Initialize a simple logging facade using the serial port.
/// 2. Setup the *Global Descriptor Table*.
/// 3. Setup the *Interrupt Descriptor Table*.
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
