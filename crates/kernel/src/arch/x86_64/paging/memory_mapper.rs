use nd_x86_64::{PageTable, PageTableEntry, PageTableFlags, PhysAddr, VirtAddr};

use super::OutOfPhysicalMemory;

/// Represents a specific map of the memory.
pub trait MemoryMap {
    /// Attempts to map the given physical address to its virtual address.
    ///
    /// If the mapping is successful, returns the virtual address of the mapping.
    ///
    /// Note that if the input address is invalid, this function might return an invalid result
    /// by extrapolating the mapping. It is up to the caller to check that the input/result is
    /// valid.
    fn physical_to_virtual(&self, physical_address: PhysAddr) -> Option<VirtAddr>;

    /// Attempts to map the given virtual address to its physical address.
    ///
    /// If the mapping is successful, returns the physical address of the mapping.
    ///
    /// Note that if the input address is invalid, this function might return an invalid result
    /// by extrapolating the mapping. It is up to the caller to check that the input/result is
    /// valid.
    fn virtual_to_physical(&self, virtual_address: VirtAddr) -> Option<PhysAddr>;
}

/// The whole physical memory is mapped to the virtual memory with an offset.
#[derive(Debug, Clone, Copy)]
pub struct OffsetMapping(VirtAddr);

impl OffsetMapping {
    /// Creates a new offset mapping with the given offset.
    ///
    /// Physical address `0` will be mapped to virtual address `offset`.
    #[inline(always)]
    pub fn new(offset: VirtAddr) -> Self {
        Self(offset)
    }
}

impl MemoryMap for OffsetMapping {
    #[inline(always)]
    fn physical_to_virtual(&self, physical_address: PhysAddr) -> Option<VirtAddr> {
        Some(self.0 + physical_address)
    }

    #[inline(always)]
    fn virtual_to_physical(&self, virtual_address: VirtAddr) -> Option<PhysAddr> {
        virtual_address.checked_sub(self.0)
    }
}

/// Represents a single entry in a page table.
pub struct MemoryMapperEntry<'a>(&'a mut PageTableEntry);

/// Allocates automatically page tables for custom mappings.
///
/// This is used to manage the memory mapping of processes.
pub struct MemoryMapper<M: MemoryMap> {
    /// The physical address of the P4 table.
    l4_table: PhysAddr,
    /// The memory mapper that's used to convert physical addresses to virtual addresses.
    mapper: M,
}

impl<M: MemoryMap> MemoryMapper<M> {
    /// Creates a new [`MemoryMapper`] instance.
    ///
    /// This function attempts to allocate a phyiscal page, and will fail if there is no more
    /// memory available.
    ///
    /// # Safety
    ///
    /// This function must be called after the page allocator has been initialized.
    ///
    /// The provided `address_space` must be the currently effective one.
    #[inline]
    pub unsafe fn new(mapper: M) -> Result<Self, OutOfPhysicalMemory> {
        let l4_table = unsafe { crate::arch::x86_64::page_allocator() }.allocate()?;

        let virt_addr = unsafe { mapper.physical_to_virtual(l4_table).unwrap_unchecked() };
        unsafe { core::ptr::write_bytes(virt_addr as *mut PageTable, 0x00, 1) };

        Ok(Self { l4_table, mapper })
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

        // SAFETY:
        //  The `unwrap_unchecked`s are always valid because we're unwraping physical addresses
        //  that have been allocated by the page allocator, which are known good addresses.
        unsafe {
            let l4_table = &mut *(self
                .mapper
                .physical_to_virtual(self.l4_table)
                .unwrap_unchecked() as *mut PageTable);
            let l4_entry = l4_table.0.get_unchecked_mut(l4_idx as usize);
            if *l4_entry == PageTableEntry::UNUSED {
                let l3_table = crate::arch::x86_64::page_allocator().allocate()?;
                *l4_entry = PageTableEntry::new(l3_table, parent_flags);
            }

            let l3_table = &mut *(self
                .mapper
                .physical_to_virtual(l4_entry.addr())
                .unwrap_unchecked() as *mut PageTable);
            let l3_entry = l3_table.0.get_unchecked_mut(l3_idx as usize);
            if *l3_entry == PageTableEntry::UNUSED {
                let l2_table = crate::arch::x86_64::page_allocator().allocate()?;
                *l3_entry = PageTableEntry::new(l2_table, parent_flags);
            }

            let l2_table = &mut *(self
                .mapper
                .physical_to_virtual(l3_entry.addr())
                .unwrap_unchecked() as *mut PageTable);
            let l2_entry = l2_table.0.get_unchecked_mut(l2_idx as usize);
            if *l2_entry == PageTableEntry::UNUSED {
                let l1_table = crate::arch::x86_64::page_allocator().allocate()?;
                *l2_entry = PageTableEntry::new(l1_table, parent_flags);
            }

            let l1_table = &mut *(self
                .mapper
                .physical_to_virtual(l2_entry.addr())
                .unwrap_unchecked() as *mut PageTable);
            let l1_entry = l1_table.0.get_unchecked_mut(l1_idx as usize);
            Ok(MemoryMapperEntry(l1_entry))
        }
    }
}

impl<M: MemoryMap> Drop for MemoryMapper<M> {
    fn drop(&mut self) {
        let page_allocator = unsafe { crate::arch::x86_64::page_allocator() };

        unsafe fn deallocate_recursive<M: MemoryMap>(
            mapper: &M,
            page_table: &PageTable,
            page_allocator: &crate::arch::x86_64::paging::PageAllocator,
            level: u32,
        ) {
            if level == 0 {
                return;
            }

            for &entry in &page_table.0 {
                if entry == PageTableEntry::UNUSED {
                    continue;
                }

                let table = unsafe {
                    &*(mapper.physical_to_virtual(entry.addr()).unwrap_unchecked()
                        as *mut PageTable)
                };

                unsafe { deallocate_recursive(mapper, table, page_allocator, level - 1) }
            }

            let phys_addr = unsafe {
                mapper
                    .virtual_to_physical(page_table as *const _ as usize as VirtAddr)
                    .unwrap_unchecked()
            };

            unsafe { page_allocator.deallocate(phys_addr) };
        }

        let l4_table = unsafe {
            &*(self
                .mapper
                .physical_to_virtual(self.l4_table)
                .unwrap_unchecked() as *mut PageTable)
        };

        unsafe { deallocate_recursive(&self.mapper, l4_table, page_allocator, 3) }
    }
}
