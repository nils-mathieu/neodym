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

pub mod raw_mapping {
    //! This module provides utility functions to create mappings without a
    //! [`MemoryMapper`](super::MemoryMapper). This is useful when a mapping needs to be created
    //! before the global page allocator has been initialized.

    #![allow(clippy::too_many_arguments)]

    use nd_x86_64::{PageTable, PageTableEntry, PageTableFlags, PhysAddr, VirtAddr};

    use super::MappingError;
    use crate::x86_64::{OutOfPhysicalMemory, PageProvider};

    /// Inserts a new entry in the page table.
    ///
    /// # Safety
    ///
    /// Given a valid physical page, `phys_to_virt` must return a valid virtual address for it.
    ///
    /// `l4` must be the physical address of a valid page table.
    ///
    /// The memory allocated by the page provider must remain valid for the lifetime `'a`.
    pub unsafe fn entry<'a>(
        l4: PhysAddr,
        provider: &PageProvider,
        phys_to_virt: &mut dyn FnMut(PhysAddr) -> VirtAddr,
        virtual_address: VirtAddr,
        parent_flags: PageTableFlags,
    ) -> Result<&'a mut PageTableEntry, OutOfPhysicalMemory> {
        let l4_idx = (virtual_address >> 39) & 0o777;
        let l3_idx = (virtual_address >> 30) & 0o777;
        let l2_idx = (virtual_address >> 21) & 0o777;
        let l1_idx = (virtual_address >> 12) & 0o777;

        let mut table = unsafe { &mut *(phys_to_virt(l4) as *mut PageTable) };
        for index in [l4_idx as usize, l3_idx as usize, l2_idx as usize] {
            let entry = unsafe { table.0.get_unchecked_mut(index) };

            table = if *entry == PageTableEntry::UNUSED {
                // The provided virtual address is not mapped. We need to allocate a page table
                // for this entry.
                let page = provider.allocate()?;
                let new_table_ptr = phys_to_virt(page) as *mut PageTable;

                // The page table is initially filled with zeroes.
                unsafe { core::ptr::write_bytes(new_table_ptr, 0x00, 1) };

                // Create the entry for the newly created page table.
                *entry = PageTableEntry::new(page, parent_flags);

                unsafe { &mut *new_table_ptr }
            } else {
                // A page table was already present as this level for the provided virtual address.
                unsafe { &mut *(phys_to_virt(entry.addr()) as *mut PageTable) }
            };
        }

        Ok(unsafe { table.0.get_unchecked_mut(l1_idx as usize) })
    }

    /// Attempts to insert a new mapping into the provided page table.
    ///
    /// # Safety
    ///
    /// Given a valid physical page, `phys_to_virt` must return a valid virtual address for it.
    ///
    /// `l4` must be the physical address of a valid page table.
    ///
    /// The memory allocated by the page provider must remain valid for the lifetime `'a`.
    pub unsafe fn create_mapping_with<'a, F>(
        l4_table: PhysAddr,
        provider: &PageProvider,
        phys_to_virt: &mut dyn FnMut(PhysAddr) -> VirtAddr,
        virt: VirtAddr,
        parent_flags: PageTableFlags,
        into_entry: F,
    ) -> Result<&'a mut PageTableEntry, MappingError>
    where
        F: FnOnce() -> Result<PageTableEntry, MappingError>,
    {
        let entry = unsafe { entry(l4_table, provider, phys_to_virt, virt, parent_flags)? };
        if *entry != PageTableEntry::UNUSED {
            return Err(MappingError::AlreadyMapped(entry.addr()));
        }

        *entry = into_entry()?;

        Ok(entry)
    }

    /// Maps `count` pages and maps them at `virt` in the given page table.
    ///
    /// # Safety
    ///
    /// Given a valid physical page, `phys_to_virt` must return a valid virtual address for it.
    ///
    /// `l4` must be the physical address of a valid page table.
    pub unsafe fn allocate_range<F>(
        l4_table: PhysAddr,
        provider: &PageProvider,
        phys_to_virt: &mut dyn FnMut(PhysAddr) -> VirtAddr,
        virt: VirtAddr,
        count: u64,
        parent_flags: PageTableFlags,
        flags: PageTableFlags,
        mut f: F,
    ) -> Result<(), MappingError>
    where
        F: FnMut(PhysAddr),
    {
        let mut virt_addr = virt;

        for _ in 0..count {
            let entry = unsafe {
                create_mapping_with(
                    l4_table,
                    provider,
                    phys_to_virt,
                    virt_addr,
                    parent_flags,
                    move || {
                        let page = provider.allocate()?;
                        Ok(PageTableEntry::new(page, flags))
                    },
                )?
            };

            f(entry.addr());
            virt_addr += 0x1000;
        }

        Ok(())
    }
}

/// The flag that we're using to determine whether a given page should be deallocated when a
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
    ) -> Result<&mut PageTableEntry, OutOfPhysicalMemory> {
        let hhdm = self.page_allocator.sys_info().hhdm_offset;

        // Those are the flags that we'll give to non-leaf entries.
        let parent_flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE
            | OWNED;

        unsafe {
            raw_mapping::entry(
                self.l4_table,
                self.page_allocator.page_provider(),
                &mut move |phys| phys + hhdm,
                virtual_address,
                parent_flags,
            )
        }
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
