use nd_x86_64::{PhysAddr, VirtAddr};

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

/// Initializes the paging system of the kernel.
///
/// The interface provided by this module can be used to manipulates the memory management unit of
/// the CPU.
///
/// # Safety
///
/// This function expects to be called only once.
///
/// Note that this function will take ownership of all provided useable memory regions. This means
/// that accessing those regions after this function has been called without first going through
/// memory management functions may result in undefined behavior. Note that the provided iterator
/// may reference data within usable memory. It will be consumed before the memory manager
/// initializes itself.
pub unsafe fn initialize_paging(usable: impl Iterator<Item = MemorySegment>) {
    for seg in usable {
        nd_log::trace!("{:?}", seg);
    }
}

/// Maps the given physical address to any avaialble virtual address.
///
/// If the physical address is already mapped to a virtual address, this function simply returns
/// that address.
///
/// # Safety
///
pub unsafe fn physical_to_virtual(physical: PhysAddr) -> VirtAddr {
    todo!();
}
