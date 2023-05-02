use bitflags::bitflags;
use core::fmt;

use crate::PhysAddr;

bitflags! {
    /// Some flags for a page table entry.
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        /// Specifies whether the mapped page frame is present.
        const PRESENT = 1 << 0;
        /// Specifies whether the page is writable. If this bit is clear, the page may only be read
        /// from. Otherwise, it can be both read and written to.
        ///
        /// By default, this only applies to rings other than ring 0, unless the **WP** bit of the
        /// **CR0** register is cleared.
        const WRITABLE = 1 << 1;
        /// Specifies whether the page is accessible from ring 3. If this bit is set, the page can
        /// be accessed from ring 3. Otherwise, it can only be accessed by the supervisor.
        const USER_ACCESSIBLE = 1 << 2;
        /// Enables write-through caching for the page.
        const WRITE_THROUGH = 1 << 3;
        /// If the bit is set, the page will not be cached. Otherwise, it will be.
        const CACHE_DISABLED = 1 << 4;
        /// This bit is automatically set by the CPU when software accesses the page.
        const ACCESSED = 1 << 5;
        /// This bit is automatically set by the CPU when software has written to the page.
        const DIRTY = 1 << 6;
        /// The entry maps a page of 4 MiB in size, rather than 4 KiB.
        const HUGE_PAGE = 1 << 7;
        /// Indicates that the *Translation Lookaside Buffer* entry for the page should not be
        /// invalidated when the CR3 register is reset.
        ///
        /// This is useful if the page is mapped into the whole virtual address space to the same
        /// physical page.
        ///
        /// The bit [`Cr4::PAGE_GLOBAL`] of the **CR4** register must be set to use global pages.
        const GLOBAL = 1 << 8;
        /// Indicates that the page cannot be used for executing code.alloc
        ///
        /// This bit is only valid if the `NXE` bit of the **EFER** register is set.
        const NO_EXECUTE = 1 << 63;
    }
}
/// A 64-bit [`PageTable`] entry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Creates a new null [`PageTableEntry`].
    pub const UNUSED: Self = Self(0);

    /// Creates a new [`PageTableEntry`].
    ///
    /// # Notes
    ///
    /// The given address must be aligned to a page boundary (4 KiB), or its lower bits will be
    /// mixed-up with the flags.
    #[inline(always)]
    pub const fn new(addr: PhysAddr, flags: PageTableFlags) -> Self {
        debug_assert!(
            addr & 0x000f_ffff_ffff_f000 == addr,
            "address must be aligned to a page boundary"
        );

        Self(addr | flags.bits())
    }

    /// Returns the physical address specified by this entry.
    #[inline(always)]
    pub const fn addr(self) -> PhysAddr {
        self.0 & 0x000f_ffff_ffff_f000
    }

    /// Returns the flags of this entry.
    #[inline(always)]
    pub const fn flags(self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            write!(f, "PageTableEntry::NULL")
        } else {
            f.debug_struct("PageTableEntry")
                .field("addr", &self.addr())
                .field("flags", &self.flags())
                .finish()
        }
    }
}

/// A 64-bit page table.
///
/// This is a simple wrapper around an array of [`PageTableEntry`]s that's aligned to a page
/// boundary.
#[repr(align(4096))]
#[derive(Debug, Clone, Copy)]
pub struct PageTable(pub [PageTableEntry; 512]);

impl PageTable {
    /// Creates a new empty page table.
    pub const fn new() -> Self {
        PageTable([PageTableEntry::UNUSED; 512])
    }
}