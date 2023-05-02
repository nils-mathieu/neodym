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

use core::num::NonZeroUsize;

pub mod raw;
pub mod sched;

/// A handle to a process.
///
/// Processes are identified by a unique handle by the kernel. This handle is used to interact with
/// the process, for example to initate inter-process communication or to terminate the process.
pub type ProcessHandle = NonZeroUsize;
