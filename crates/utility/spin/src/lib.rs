//! Spinlock-based synchronization primitives.

#![no_std]

mod mutex;

pub use self::mutex::*;
