//! This module contains the *Interrupt Service Routine* which will be called when interrupts
//! are received.

use nd_x86_64::InterruptStackFrame;

mod errors;

pub use self::errors::*;

pub extern "x86-interrupt" fn breakpoint(_: InterruptStackFrame) {
    nd_log::info!("BREAKPOINT");
}
