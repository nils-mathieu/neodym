use core::mem::MaybeUninit;

use nd_x86_64::{Cr3, Cr3Flags, PageTable, PageTableFlags, PhysAddr, VirtAddr};
use neodym_sys_common::PageSize;

use crate::x86_64::SysInfoTok;

use super::mapping::MappingError;
use super::{OutOfPhysicalMemory, PageAllocatorTok};

/// The bit to enable to indicate that a page is owned by the current process. This means that
/// the pages used to map in virtual memory should be deallocated when the process is destroyed.
const OWNED: PageTableFlags = PageTableFlags::USER_0;

/// Offsets a physical addres by the HHDM start address.
///
/// # Safety
///
/// The global SysInfo structure must be initialized.
///
/// This function cannot be `unsafe` because it must be coerced to a function pointer.
#[inline]
fn offset_by_hhdm(page: PhysAddr) -> VirtAddr {
    // SAFETY:
    //  This function is only defined in this module, and won't be used in a context were the
    //  SysInfo global structure is not yet initialized.
    let sys_info = unsafe { SysInfoTok::unchecked() };

    sys_info.hhdm_start + page
}

/// A virtual address space that keeps track of which pages are owned by the current process and
/// deallocates them when the process is destroyed.
pub struct OwnedMapper {
    pml4: PhysAddr,
    page_allocator: PageAllocatorTok,
}

impl OwnedMapper {
    /// Creates a new [`OwnedMapper`] instance.
    pub fn new(page_allocator: PageAllocatorTok) -> Result<Self, OutOfPhysicalMemory> {
        let pml4 = page_allocator.allocate()?;

        unsafe {
            core::ptr::write_bytes(
                (pml4 + page_allocator.sys_info().hhdm_start) as *mut u8,
                0,
                0x1000,
            );
        }

        Ok(Self {
            pml4,
            page_allocator,
        })
    }

    /// Returns a reference to the PML4 page table.
    #[inline(always)]
    pub fn pml4_mut(&mut self) -> &mut PageTable {
        unsafe { &mut *((self.pml4 + self.page_allocator.sys_info().hhdm_start) as *mut PageTable) }
    }

    /// Loads this address space into the CPU.
    ///
    /// # Safety
    ///
    /// Very unsafe, yes.
    #[inline(always)]
    pub unsafe fn switch(&self) {
        unsafe { nd_x86_64::set_cr3(Cr3::new(self.pml4, Cr3Flags::empty())) };
    }

    /// Allocates a new page and maps it into the current address space.
    pub fn allocate_mapping(
        &mut self,
        virt: VirtAddr,
        parent_flags: PageTableFlags,
        flags: PageTableFlags,
    ) -> Result<PhysAddr, MappingError> {
        let phys = self.page_allocator.allocate()?;

        crate::x86_64::mapping::map_4k(
            self.pml4,
            self.page_allocator.page_provider(),
            &mut offset_by_hhdm,
            virt,
            phys,
            parent_flags | OWNED,
            flags | OWNED,
        )?;

        Ok(phys)
    }

    /// Allocates physical pages and calls the provided callback with a mutable slice of
    /// [`MaybeUninit<u8>`]s.
    ///
    /// # Arguments
    ///
    /// The `virt` argument is the virtual address to map the pages into.
    ///
    /// The `count` argument is the number of pages to allocate.
    ///
    /// The `with` argument is the callback to call on each allocated pages.
    ///
    /// # Notes
    ///
    /// If an error occurs, the pages that were allocated successfully are not deallocated until
    /// the [`OwnedMapper`] instance is dropped.
    pub fn load_with<F>(
        &mut self,
        mut virt: VirtAddr,
        count: u64,
        flags: PageTableFlags,
        parent_flags: PageTableFlags,
        mut with: F,
    ) -> Result<(), MappingError>
    where
        F: FnMut(&mut [MaybeUninit<u8>]),
    {
        for _ in 0..count {
            let phys = self.allocate_mapping(virt, parent_flags, flags)?;

            let in_kernel_addr_space = self.page_allocator.sys_info().hhdm_start + phys;
            let in_kernel_addr_space = unsafe {
                core::slice::from_raw_parts_mut(
                    in_kernel_addr_space as *mut MaybeUninit<u8>,
                    0x1000,
                )
            };

            with(in_kernel_addr_space);

            virt += 0x1000;
        }

        Ok(())
    }

    /// Loads the provided data into the address space at the provided virtual address.
    pub fn load(
        &mut self,
        virt: VirtAddr,
        mut data: &[u8],
        flags: PageTableFlags,
        parent_flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        let count = (data.len() as u64 + 0xFFF) / 0x1000;
        self.load_with(virt, count, flags, parent_flags, |page| unsafe {
            let to_copy = if data.len() > 0x1000 {
                0x1000
            } else {
                data.len()
            };

            core::ptr::copy_nonoverlapping(data.as_ptr(), page.as_mut_ptr() as *mut u8, to_copy);
            data = data.get_unchecked(to_copy..);
        })
    }

    /// Loads the requested number of pages into the address space at the provided virtual address.
    pub fn load_uninit(
        &mut self,
        virt: VirtAddr,
        count: u64,
        flags: PageTableFlags,
        parent_flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        self.load_with(virt, count, flags, parent_flags, |_| ())
    }
}
