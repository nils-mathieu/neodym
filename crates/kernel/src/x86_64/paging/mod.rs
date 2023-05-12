use core::alloc::AllocError;

use nd_x86_64::PhysAddr;

mod identity_map;
mod page_allocator;
mod page_provider;

pub use self::identity_map::*;
pub use self::page_allocator::*;
pub use self::page_provider::*;

/// The system is out of available physical memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfPhysicalMemory;

impl From<OutOfPhysicalMemory> for AllocError {
    #[inline(always)]
    fn from(_: OutOfPhysicalMemory) -> Self {
        AllocError
    }
}

/// A memory segment that is useable by the kernel.
#[derive(Debug, Clone, Copy)]
pub struct MemorySegment {
    /// The base address of the segment.
    ///
    /// This is a *physcal* address.
    pub base: PhysAddr,
    /// The size of the segment.
    pub length: u64,
}
