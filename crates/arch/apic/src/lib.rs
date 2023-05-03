//! An interface to the APIC interrupt architecture.

#![no_std]

mod lapic;

pub use self::lapic::*;
