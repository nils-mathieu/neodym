//! Code specific to the `x86_64` CPU architecture.

mod apic;
mod interrupts;
mod kernel_info;
mod logger;
mod paging;
mod process;
mod tables;

pub use self::apic::*;
pub use self::kernel_info::*;
pub use self::logger::*;
pub use self::paging::*;
pub use self::process::*;
pub use self::tables::*;

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
