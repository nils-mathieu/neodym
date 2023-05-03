use core::mem::size_of;

use nd_array::Vec;
use nd_x86_64::{PageTable, PageTableFlags, PhysAddr, VirtAddr};

use super::PhysicalFrame;

/// An error that can occur when mapping memory.
pub enum MemoryMapError {
    /// The provided virtual address is already mapped to a physical address.
    AlreadyMapped,
    /// There is no more physical memory available.
    OutOfMemory,
}

/// A list of physical frames used by a [`MemoryMapper`].
struct PhysicalFrames {
    frames: Vec<PhysicalFrame, { (4096 - size_of::<usize>()) / size_of::<PhysicalFrame>() }>,
}

const _: () = assert!(size_of::<PhysicalFrames>() == 4096);

/// Wraps a bunch of page tables and provides methods to map physical memory to virtual memory.
pub struct MemoryMapper {
    /// A reference to the L4 page table.
    ///
    /// This pointer maps to the physical frame `l4_frame` and will therefor remain valid because
    /// `l4_frame` is guards the allocation.
    l4: *mut PageTable,
}

impl MemoryMapper {
    /// Creates a new [`MemoryMapper`] instance for the provided page table.
    ///
    /// # Safety
    ///
    /// `l4` must map to the provided l4_frame.
    pub unsafe fn new(l4_frame: PhysicalFrame, l4: *mut PageTable) -> Self {
        todo!();
    }

    /// Maps a specific physical address to a specific virtual address.
    ///
    /// # Safety
    ///
    /// The provided physical page must currently be unused and must be aligned to a 4096-byte
    /// boundary.
    ///
    /// The provided virtual address must be aligned to a 4096-byte boundary.
    ///
    /// The global page allocator must be initialized.
    pub unsafe fn map_specific(
        &mut self,
        phys: PhysAddr,
        virt: VirtAddr,
    ) -> Result<(), MemoryMapError> {
        debug_assert!(phys % 4096 == 0);
        debug_assert!(virt % 4096 == 0);

        let indexes = [
            (virt >> 39) & 0o777,
            (virt >> 30) & 0o777,
            (virt >> 21) & 0o777,
            (virt >> 12) & 0o777,
        ];

        todo!();
    }

    /// Creates a new memory mapping for the provided virtual address. A physical page is allocated
    /// and mapped to the provided virtual address.
    ///
    /// # Safety
    ///
    /// The provided virtual address must be aligned to a 4096-byte boundary.
    ///
    /// The global page allocator must be initialized.
    pub unsafe fn map(&mut self, virt: VirtAddr) -> Result<PhysicalFrame, MemoryMapError> {
        let page = unsafe { PhysicalFrame::allocate() }.ok_or(MemoryMapError::OutOfMemory)?;
        match unsafe { self.map_specific(page.addr(), virt) } {
            Ok(()) => Ok(page),
            Err(err) => Err(err),
        }
    }
}
