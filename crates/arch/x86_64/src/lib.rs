//! Provide a more "rusty" interface to x86_64-specific instructions, registers, and structures.
//!
//! This crate does not aim to make those operations safer, but simply to make them easier to use
//! and manipulate.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

mod instructions;

pub use self::instructions::*;
