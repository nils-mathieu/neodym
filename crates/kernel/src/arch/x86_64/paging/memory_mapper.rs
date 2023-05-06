use nd_x86_64::{PageTable, PageTableEntry, PageTableFlags, PhysAddr, VirtAddr};

use super::OutOfPhysicalMemory;
use crate::arch::x86_64::PageAllocator;

/// An error which can occur when mapping a page.
pub enum MappingError {
    /// The request virtual address is already mapped to a physical page.
    AlreadyMapped(PhysAddr),
    /// The page allocator is out of physical memory.
    OutOfPhysicalMemory,
}

impl From<OutOfPhysicalMemory> for MappingError {
    #[inline(always)]
    fn from(_: OutOfPhysicalMemory) -> Self {
        Self::OutOfPhysicalMemory
    }
}

/// Represents a leaf entry in a page table.
#[repr(transparent)]
pub struct MemoryMapperEntry(PageTableEntry);

impl MemoryMapperEntry {
    /// Creates a new [`MemoryMapperEntry`] instance.
    #[inline(always)]
    fn new_mut(inner: &mut PageTableEntry) -> &mut Self {
        unsafe { &mut *(inner as *mut PageTableEntry as *mut Self) }
    }

    /// Returns the virtual address of the page within the kernel address space.
    #[inline]
    pub fn kernel_virtual_address(&mut self) -> VirtAddr {
        let page_allocator = unsafe { crate::arch::x86_64::page_allocator() };
        page_allocator.physical_to_virtual(self.0.addr())
    }
}

/// Allocates automatically page tables for custom mappings.
///
/// This is used to manage the memory mapping of processes.
pub struct MemoryMapper {
    /// The physical address of the P4 table.
    l4_table: PhysAddr,
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
            l4_table: unsafe { crate::arch::x86_64::page_allocator() }.allocate()?,
        })
    }

    /// Returns an entry within the whole page table.
    ///
    /// If the entry does not exist yet, it is created automatically.
    ///
    /// Note that the lower 12 bits of the provided virtual address will be ignored.
    ///
    /// Note that this function does not allocate any physical page for the final mapping.
    pub fn entry(
        &mut self,
        virtual_address: VirtAddr,
    ) -> Result<&mut PageTableEntry, OutOfPhysicalMemory> {
        let page_allocator = unsafe { crate::arch::x86_64::page_allocator() };

        // Those are the flags that we'll give to non-leaf entries.
        let parent_flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let l4_idx = (virtual_address >> 39) & 0o777;
        let l3_idx = (virtual_address >> 30) & 0o777;
        let l2_idx = (virtual_address >> 21) & 0o777;
        let l1_idx = (virtual_address >> 12) & 0o777;

        let mut table =
            unsafe { &mut *(page_allocator.physical_to_virtual(self.l4_table) as *mut PageTable) };
        for index in [l4_idx as usize, l3_idx as usize, l2_idx as usize] {
            let entry = unsafe { table.0.get_unchecked_mut(index) };

            table = if *entry == PageTableEntry::UNUSED {
                // The provided virtual address is not mapped. We need to allocate a page table
                // for this entry.
                let page = page_allocator.allocate()?;
                let new_table_ptr = page_allocator.physical_to_virtual(page) as *mut PageTable;

                // The page table is initially filled with zeroes.
                unsafe { core::ptr::write_bytes(new_table_ptr, 0x00, 1) };

                // Create the entry for the newly created page table.
                *entry = PageTableEntry::new(page, parent_flags);

                unsafe { &mut *new_table_ptr }
            } else {
                // A page table was already present as this level for the provided virtual address.
                unsafe {
                    &mut *(page_allocator.physical_to_virtual(entry.addr()) as *mut PageTable)
                }
            };
        }

        Ok(unsafe { table.0.get_unchecked_mut(l1_idx as usize) })
    }

    /// Creates a new mapping for the provided virtual address.
    pub fn create_mapping(
        &mut self,
        virt: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<&mut MemoryMapperEntry, MappingError> {
        let page_allocator = unsafe { crate::arch::x86_64::page_allocator() };

        let entry = self.entry(virt)?;

        if *entry != PageTableEntry::UNUSED {
            return Err(MappingError::AlreadyMapped(entry.addr()));
        }

        *entry = PageTableEntry::new(page_allocator.allocate()?, flags);

        Ok(MemoryMapperEntry::new_mut(entry))
    }
}

impl Drop for MemoryMapper {
    fn drop(&mut self) {
        unsafe fn drop_recursive(table: PhysAddr, page_allocator: &PageAllocator, level: usize) {
            let table =
                unsafe { &mut *(page_allocator.physical_to_virtual(table) as *mut PageTable) };

            // Level 1 page table do not have children.
            if level > 1 {
                for entry in &mut table.0 {
                    if *entry != PageTableEntry::UNUSED {
                        unsafe { drop_recursive(entry.addr(), page_allocator, level - 1) };
                    }
                }
            }

            // Deallocate the referenced page.
            unsafe { page_allocator.deallocate(table.0[0].addr()) };
        }

        let page_allocator = unsafe { crate::arch::x86_64::page_allocator() };
        unsafe { drop_recursive(self.l4_table, page_allocator, 4) };
    }
}
