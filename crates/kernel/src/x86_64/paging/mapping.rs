use nd_x86_64::{Cr3, Cr3Flags, PageTable, PageTableEntry, PageTableFlags, PhysAddr, VirtAddr};

use super::{OutOfPhysicalMemory, PageProvider};

const ONE_GIGABYTE: u64 = 512 * TWO_MEGABYTES;
const TWO_MEGABYTES: u64 = 512 * FOUR_KILOBYTES;
const FOUR_KILOBYTES: u64 = 4096;

/// Returns the index of the P4 entry for the given virtual address.
#[inline(always)]
fn pml4e_index(virt: VirtAddr) -> usize {
    ((virt >> 39) & 0o777) as usize
}

/// Returns the index of the P3 entry for the given virtual address.
#[inline(always)]
fn pdpte_index(virt: VirtAddr) -> usize {
    ((virt >> 30) & 0o777) as usize
}

/// Returns the index of the P2 entry for the given virtual address.
#[inline(always)]
fn pde_index(virt: VirtAddr) -> usize {
    ((virt >> 21) & 0o777) as usize
}

/// Returns the index of the P1 entry for the given virtual address.
#[inline(always)]
fn pte_index(virt: VirtAddr) -> usize {
    ((virt >> 12) & 0o777) as usize
}

/// An error which might occur when mapping a virtual address to a physical address.
#[derive(Debug, Clone, Copy)]
pub enum MappingError {
    /// The system is out of physical memory and cannot allocate for a new page.
    OutOfPhysicalMemory,
    /// The requested virtual address is already mapped to some physical page.
    AlreadyMapped,
}

impl From<OutOfPhysicalMemory> for MappingError {
    #[inline(always)]
    fn from(_: OutOfPhysicalMemory) -> Self {
        Self::OutOfPhysicalMemory
    }
}

/// Gets an entry into the page table; the returned entry points to a page directory which
/// references an allocated page (of potentially more directory entries, or page table entries).
///
/// # Safety
///
/// `index` must be less than 512.
unsafe fn get_directory_entry<'a>(
    page_table: PhysAddr,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    provider: &PageProvider,
    index: usize,
) -> Result<&'a mut PageTableEntry, MappingError> {
    debug_assert!(index < 512);

    let table = unsafe { &mut *(map(page_table) as *mut PageTable) };
    let entry = unsafe { table.get_unchecked_mut(index) };
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let phys_addr = provider.allocate()?;

        unsafe {
            core::ptr::write_bytes(map(phys_addr) as *mut u8, 0, FOUR_KILOBYTES as usize);
        }

        *entry = PageTableEntry::new(
            phys_addr,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        Ok(entry)
    } else if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        Err(MappingError::AlreadyMapped)
    } else {
        Ok(entry)
    }
}

/// Gets a page entry.
///
/// The returned entry is unused.
///
/// # Safety
///
/// `index` must be less than 512.
unsafe fn get_page_entry<'a>(
    page_table: PhysAddr,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    index: usize,
) -> Result<&'a mut PageTableEntry, MappingError> {
    debug_assert!(index < 512);

    let table = unsafe { &mut *(map(page_table) as *mut PageTable) };
    let entry = unsafe { table.get_unchecked_mut(index) };
    if entry.flags().contains(PageTableFlags::PRESENT) {
        Err(MappingError::AlreadyMapped)
    } else {
        Ok(entry)
    }
}

/// Maps the provided virtual address to the provided physical address.
///
/// # Arguments
///
/// Both `virt_addr` and `phys_addr` must be aligned to 1 GiB.
fn map_1g(
    p4: PhysAddr,
    provider: &PageProvider,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    virt_addr: VirtAddr,
    phys_addr: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), MappingError> {
    debug_assert!(virt_addr % ONE_GIGABYTE == 0);
    debug_assert!(phys_addr % ONE_GIGABYTE == 0);

    let pml4e = unsafe { get_directory_entry(p4, map, provider, pml4e_index(virt_addr))? };
    let pdpte = unsafe { get_page_entry(pml4e.addr(), map, pdpte_index(virt_addr))? };

    *pdpte = PageTableEntry::new(phys_addr, flags | PageTableFlags::HUGE_PAGE);

    Ok(())
}

/// Maps the provided virtual address to the provided physical address using 4 MiB pages.
///
/// # Arguments
///
/// Both `virt_addr` and `phys_addr` must be aligned to 2 MiB.
fn map_2m(
    l4: PhysAddr,
    provider: &PageProvider,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    virt_addr: VirtAddr,
    phys_addr: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), MappingError> {
    debug_assert!(virt_addr % TWO_MEGABYTES == 0);
    debug_assert!(phys_addr % TWO_MEGABYTES == 0);

    let pml4e = unsafe { get_directory_entry(l4, map, provider, pml4e_index(virt_addr))? };
    let pdpte =
        unsafe { get_directory_entry(pml4e.addr(), map, provider, pdpte_index(virt_addr))? };
    let pde = unsafe { get_page_entry(pdpte.addr(), map, pde_index(virt_addr))? };

    *pde = PageTableEntry::new(phys_addr, flags | PageTableFlags::HUGE_PAGE);

    Ok(())
}

/// Maps the provided virtual address to the provided physical address using 4 KiB pages.
///
/// # Arguments
///
/// Both `virt_addr` and `phys_addr` must be aligned to 4 KiB.
fn map_4k(
    pml4: PhysAddr,
    provider: &PageProvider,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    virt_addr: VirtAddr,
    phys_addr: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), MappingError> {
    debug_assert!(virt_addr % FOUR_KILOBYTES == 0);
    debug_assert!(phys_addr % FOUR_KILOBYTES == 0);

    let pml4e = unsafe { get_directory_entry(pml4, map, provider, pml4e_index(virt_addr))? };
    let pdpte =
        unsafe { get_directory_entry(pml4e.addr(), map, provider, pdpte_index(virt_addr))? };
    let pde = unsafe { get_directory_entry(pdpte.addr(), map, provider, pde_index(virt_addr))? };
    let pte = unsafe { get_page_entry(pde.addr(), map, pte_index(virt_addr))? };

    *pte = PageTableEntry::new(phys_addr, flags);

    Ok(())
}

/// Maps the provided physical addresses to the provided virtual addresses.
fn map_range(
    l4: PhysAddr,
    provider: &PageProvider,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    mut virt_addr: VirtAddr,
    mut phys_addr: PhysAddr,
    mut amount: u64,
    flags: PageTableFlags,
) -> Result<(), MappingError> {
    while amount != 0 {
        if amount >= ONE_GIGABYTE {
            map_1g(l4, provider, map, virt_addr, phys_addr, flags)?;

            amount -= ONE_GIGABYTE;
            virt_addr += ONE_GIGABYTE;
            phys_addr += ONE_GIGABYTE;
        } else if amount >= TWO_MEGABYTES {
            map_2m(l4, provider, map, virt_addr, phys_addr, flags)?;

            amount -= TWO_MEGABYTES;
            virt_addr += TWO_MEGABYTES;
            phys_addr += TWO_MEGABYTES;
        } else {
            map_4k(l4, provider, map, virt_addr, phys_addr, flags)?;

            amount = amount.saturating_sub(FOUR_KILOBYTES);
            virt_addr += FOUR_KILOBYTES;
            phys_addr += FOUR_KILOBYTES;
        }
    }

    Ok(())
}

/// Sets an identiy map for the given L4 page table.
///
/// - Memory from 0x0 to `upper_bound` is identity mapped.
/// - The kernel is mapped at `0xFFFF_FFFF_8000_0000`.
///
/// # Errors
///
/// In case of error, this function leaks memory as it has no way to free allocated pages.
///
/// # Safety
///
///
/// Changing the page table is unsafe.
///
/// - This function should probably be called only once?
/// - The kernel must've been compiled to be mapped at `kernel_virt`.
pub unsafe fn setup_paging(
    provider: &PageProvider,
    map: &mut dyn FnMut(PhysAddr) -> VirtAddr,
    upper_bound: PhysAddr,
    kernel_phys: PhysAddr,
    kernel_virt: VirtAddr,
    kernel_size: u64,
) -> Result<(), MappingError> {
    nd_log::trace!("Setting up virtual memory...");
    let pml4 = provider.allocate()?;

    unsafe {
        core::ptr::write_bytes(map(pml4) as *mut u8, 0, FOUR_KILOBYTES as usize);
    }

    //
    // IDENTITY MAPPING
    //
    nd_log::trace!(
        "  > Identity mapping physical memory up to {:#x}...",
        upper_bound
    );
    map_range(
        pml4,
        provider,
        map,
        0,
        0,
        upper_bound,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    )?;

    //
    // MAP THE KERNEL AT THE REQUESTED ADDRESS
    //
    nd_log::trace!(
        "  > Mapping kernel from {:#x} to {:#x}...",
        kernel_phys,
        kernel_virt
    );
    map_range(
        pml4,
        provider,
        map,
        kernel_virt,
        kernel_phys,
        kernel_size,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
    )?;

    nd_log::trace!("  > Switching address space...");
    unsafe {
        nd_x86_64::set_cr3(Cr3::new(pml4, Cr3Flags::empty()));
    }

    Ok(())
}
