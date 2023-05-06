use core::marker::PhantomData;

use nd_x86_64::{PageTableEntry, PageTableFlags, VirtAddr};

use crate::allocator::paging::PageBox;

use super::OutOfPhysicalMemory;

/// Represents a leaf entry in a page table.
pub struct MemoryMapperEntry<'a>(&'a mut PageTableEntry);

/// A page table that owns the memory pages of its entries.
#[repr(transparent)]
struct PageTable {
    inner: nd_x86_64::PageTable,

    /// This is used by the drop checker to understand that we will drop those boxes.
    _marker: PhantomData<PageBox<PageTable>>,
}

/// Allocates automatically page tables for custom mappings.
///
/// This is used to manage the memory mapping of processes.
pub struct MemoryMapper {
    /// The physical address of the P4 table.
    l4_table: PageBox<PageTable>,
}

impl MemoryMapper {
    /// Creates a new [`MemoryMapper`] instance.
    ///
    /// This function attempts to allocate a phyiscal page, and will fail if there is no more
    /// memory available.
    ///
    /// # Safety
    ///
    /// This function must be called after the page allocator has been initialized.
    #[inline]
    pub unsafe fn new() -> Result<Self, OutOfPhysicalMemory> {
        Ok(Self {
            l4_table: unsafe { PageBox::zeroed()? },
        })
    }

    /// Returns an entry within the whole page table.
    ///
    /// If the entry does not exist yet, it is created automatically.
    ///
    /// # Safety
    ///
    /// `address_space` must accurately return the virtual address associated with the given
    /// physical address.
    pub unsafe fn entry(
        &mut self,
        virtual_address: VirtAddr,
    ) -> Result<MemoryMapperEntry, OutOfPhysicalMemory> {
        // Those are the flags that we'll give to non-leaf entries.
        let parent_flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let l4_idx = (virtual_address >> 39) & 0o777;
        let l3_idx = (virtual_address >> 30) & 0o777;
        let l2_idx = (virtual_address >> 21) & 0o777;
        let l1_idx = (virtual_address >> 12) & 0o777;

        todo!();
    }
}

impl Drop for MemoryMapper {
    fn drop(&mut self) {
        todo!();
    }
}
