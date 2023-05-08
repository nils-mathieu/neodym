//! This module contains the paging code for the `x86_64` CPU architecture.
//!
//! This is mainly utility types and functions to manage the page tables, memory mapping and
//! address translations.
//!
//! # Paging Model On x86_64
//!
//! On x86_64, the kernel assumes that a higher half direct map is setup, only accessible by the
//! kernel. This means that the kernel can access all physical memory using the same virtual
//! addresses.

mod memory_mapper;
mod page_allocator;
mod page_box;
mod page_list;

pub use self::memory_mapper::*;
pub use self::page_allocator::*;
pub use self::page_box::*;
pub use self::page_list::*;

/// The size of a physical page.
pub const PAGE_SIZE: usize = 4096;
