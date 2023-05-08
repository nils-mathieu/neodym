//! This libraries defines wrapper function for the system calls defined by the Neodym Operating
//! System.
//!
//! # Portablity
//!
//! Because system calls are fundamentally architecture dependent, some of the functionalities
//! defined in this crate might be conditionally compiled depending on the target architecture.
//!
//! At the moment, only the `x86_64` architecture is supported.

#![no_std]

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

pub use neodym_sys_common::*;

/// A handle to a process.
pub type ProcessHandle = core::num::NonZeroUsize;
