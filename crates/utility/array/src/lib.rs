//! Array-based data structures.

#![no_std]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod binary_heap;
mod slab;
mod string;
mod vec;

pub use self::binary_heap::*;
pub use self::slab::*;
pub use self::string::*;
pub use self::vec::*;
