//! Provide a more "rusty" interface to x86_64-specific instructions, registers, and structures.
//!
//! This crate does not aim to make those operations safer, but simply to make them easier to use
//! and manipulate.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod gdt;
mod idt;
mod instructions;

pub use self::gdt::*;
pub use self::idt::*;
pub use self::instructions::*;

/// A virtual address.
pub type VirtAddr = u64;

/// A physical address.
pub type PhysAddr = u64;

/// A privilege level (i.e. ring level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
pub enum PrivilegeLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

impl PrivilegeLevel {
    /// Creates a new [`PrivilegeLevel`] from the provided level.
    ///
    /// # Safety
    ///
    /// `level` must be less or equal to `3`.
    ///
    /// # Panics
    ///
    /// In debug mode, this function panics if the provided level is invalid (i.e. greater than 3).
    #[inline(always)]
    pub const unsafe fn from_raw_unchecked(level: u8) -> Self {
        unsafe { core::mem::transmute(level) }
    }
}
