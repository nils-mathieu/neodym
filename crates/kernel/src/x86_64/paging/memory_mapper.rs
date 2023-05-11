use core::ops::{Deref, DerefMut};

use nd_x86_64::{Cr3, Cr3Flags, PageTable, PageTableEntry, PageTableFlags, PhysAddr, VirtAddr};

use super::{OutOfPhysicalMemory, PageAllocatorTok};

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
pub struct MemoryMapperEntry {
    entry: PageTableEntry,
    allocator: PageAllocatorTok,
}

impl MemoryMapperEntry {
    /// Creates a new [`MemoryMapperEntry`] instance.
    #[inline(always)]
    fn new_mut(inner: &mut PageTableEntry) -> &mut Self {
        unsafe { &mut *(inner as *mut PageTableEntry as *mut Self) }
    }

    /// Returns the virtual address of the page within the kernel address space.
    #[inline(always)]
    pub fn kernel_virtual_address(&mut self) -> VirtAddr {
        self.entry.addr() + self.allocator.sys_info().hhdm_offset
    }
}

impl Deref for MemoryMapperEntry {
    type Target = PageTableEntry;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl DerefMut for MemoryMapperEntry {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}

/// The flag that we're using to determine whether a given page should be deallocate when a
/// [`MemoryMapper`] is dropped.
const OWNED: PageTableFlags = PageTableFlags::USER_0;

/// Represents an address space (of a userspace process for example).
///
/// This type provides multiple convenience methods to interact with an owned page table. Creating
/// new pages, mapping them, etc. Note that by default, created pages are owned by the mapper, and
/// will be deallocated when the mapper is dropped.
///
/// # Conditional Ownership
///
/// Some bits of the table entry flags can be defined by the user (us). We're using the first one
/// to determine whether we should deallocate the page when the `MemoryMapper` is dropped or not.
/// Note that this process is recursive, meaning that if a page is not owned, then none of its
/// children can be owned either.
///
/// This is mainly used for the kernel's address space, where we don't want to deallocate the
/// pages.
pub struct MemoryMapper {
    /// The physical address of the P4 table.
    l4_table: PhysAddr,
    page_allocator: PageAllocatorTok,
}

impl MemoryMapper {
    /// Creates a new [`MemoryMapper`] instance.
    ///
    /// This function attempts to allocate a phyiscal page, and will fail if there is no more
    /// memory available.
    #[inline]
    pub fn new(page_allocator: PageAllocatorTok) -> Result<Self, OutOfPhysicalMemory> {
        let l4_table = page_allocator.allocate()?;

        unsafe {
            core::ptr::write_bytes(
                (l4_table + page_allocator.sys_info().hhdm_offset) as *mut PageTable,
                0x00,
                1,
            );
        }

        Ok(Self {
            l4_table,
            page_allocator,
        })
    }

    /// Switches the current address space to the one represented by this [`MemoryMapper`].
    ///
    /// # Safety
    ///
    /// Very unsafe, yes.
    #[inline]
    pub unsafe fn switch(&self) {
        unsafe {
            nd_x86_64::set_cr3(Cr3::new(self.l4_table, Cr3Flags::empty()));
        }
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
    ) -> Result<&mut MemoryMapperEntry, OutOfPhysicalMemory> {
        let hhdm = self.page_allocator.sys_info().hhdm_offset;

        // Those are the flags that we'll give to non-leaf entries.
        let parent_flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE
            | OWNED;

        let l4_idx = (virtual_address >> 39) & 0o777;
        let l3_idx = (virtual_address >> 30) & 0o777;
        let l2_idx = (virtual_address >> 21) & 0o777;
        let l1_idx = (virtual_address >> 12) & 0o777;

        let mut table = unsafe { &mut *((self.l4_table + hhdm) as *mut PageTable) };
        for index in [l4_idx as usize, l3_idx as usize, l2_idx as usize] {
            let entry = unsafe { table.0.get_unchecked_mut(index) };

            table = if *entry == PageTableEntry::UNUSED {
                // The provided virtual address is not mapped. We need to allocate a page table
                // for this entry.
                let page = self.page_allocator.allocate()?;
                let new_table_ptr = (page + hhdm) as *mut PageTable;

                // The page table is initially filled with zeroes.
                unsafe { core::ptr::write_bytes(new_table_ptr, 0x00, 1) };

                // Create the entry for the newly created page table.
                *entry = PageTableEntry::new(page, parent_flags);

                unsafe { &mut *new_table_ptr }
            } else {
                // A page table was already present as this level for the provided virtual address.
                unsafe { &mut *((entry.addr() + hhdm) as *mut PageTable) }
            };
        }

        Ok(MemoryMapperEntry::new_mut(unsafe {
            table.0.get_unchecked_mut(l1_idx as usize)
        }))
    }

    /// Creates a new mapping for the provided virtual address.
    ///
    /// This function does not allocate any physical page for the final mapping.
    pub fn create_mapping(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PageTableFlags,
    ) -> Result<&mut MemoryMapperEntry, MappingError> {
        let entry = self.entry(virt)?;
        if **entry != PageTableEntry::UNUSED {
            return Err(MappingError::AlreadyMapped(entry.addr()));
        }

        **entry = PageTableEntry::new(phys, flags);

        Ok(entry)
    }

    /// Creates a new mapping for the provided virtual address.
    pub fn allocate_mapping(
        &mut self,
        virt: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<&mut MemoryMapperEntry, MappingError> {
        let page_allocator = self.page_allocator;

        let entry = self.entry(virt)?;
        if **entry != PageTableEntry::UNUSED {
            return Err(MappingError::AlreadyMapped(entry.addr()));
        }

        **entry = PageTableEntry::new(page_allocator.allocate()?, flags | OWNED);

        Ok(entry)
    }

    /// Loads the requested number of pages starting at the provided virtual address. On each
    /// page, the provided function is called with the entry for the page.
    pub fn load_at_with<F>(
        &mut self,
        at: VirtAddr,
        count: u64,
        flags: PageTableFlags,
        mut with: F,
    ) -> Result<(), MappingError>
    where
        F: FnMut(&mut MemoryMapperEntry),
    {
        let mut virt_addr = at;

        for _ in 0..count {
            let page = self.allocate_mapping(virt_addr, flags)?;
            with(page);
            virt_addr += 0x1000;
        }

        Ok(())
    }

    /// Loads the provided data at the provided virtual address.
    ///
    /// This function allocates physical pages as needed. Note that those physical pages are not
    /// guarenteed to be contiguous.
    pub fn load_at(
        &mut self,
        data: &[u8],
        at: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        let count = (data.len() + 0xfff) / 0x1000;
        let mut remainder = data;

        self.load_at_with(at, count as u64, flags, |page| unsafe {
            let to_copy = core::cmp::min(remainder.len(), 0x1000);

            // SAFETY:
            //  We're copying a chunk of the input data into the page.
            core::ptr::copy_nonoverlapping(
                remainder.as_ptr(),
                page.kernel_virtual_address() as usize as *mut u8,
                to_copy,
            );

            remainder = remainder.get_unchecked(to_copy..);
        })
    }

    /// Maps the kernel into this address space.
    pub fn map_kernel(&mut self) -> Result<(), MappingError> {
        let info = self.page_allocator.sys_info();

        let mut phys_addr = info.kernel_phys_addr;
        let mut virt_addr = info.kernel_virt_addr;
        let mut remainder = info.kernel_virt_end_addr - info.kernel_virt_addr;

        while remainder > 0 {
            let size = core::cmp::min(remainder, 0x1000);
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            self.create_mapping(virt_addr, phys_addr, flags)?;

            phys_addr += size;
            virt_addr += size;
            remainder -= size;
        }

        Ok(())
    }
}

impl Drop for MemoryMapper {
    fn drop(&mut self) {
        unsafe fn drop_recursive(table: PhysAddr, page_allocator: PageAllocatorTok, level: usize) {
            let hhdm = page_allocator.sys_info().hhdm_offset;
            let table = unsafe { &mut *((table + hhdm) as *mut PageTable) };

            // Level 1 page table do not have children.
            if level > 1 {
                for entry in &mut table.0 {
                    if *entry != PageTableEntry::UNUSED && entry.flags().contains(OWNED) {
                        unsafe { drop_recursive(entry.addr(), page_allocator, level - 1) };
                    }
                }
            }

            // Deallocate the referenced page.
            unsafe { page_allocator.deallocate(table.0[0].addr()) };
        }

        unsafe { drop_recursive(self.l4_table, self.page_allocator, 4) };
    }
}
