//! Structures and constants specific to x86_64.

/// A system call supported on the x86_64 architecture.
///
/// The disciminant of this enum corresponds to the system call number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum SystemCall {
    Terminate,
    MapMemory,
}

impl SystemCall {
    /// The number of defined system calls.
    pub const COUNT: usize = 1;

    /// Creates a new [`SystemCall`] from a system call number.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the system call number is valid.
    #[inline(always)]
    pub const unsafe fn from_usize_unchecked(n: usize) -> Self {
        unsafe { core::mem::transmute(n) }
    }

    /// Creates a new [`SystemCall`] from a system call number.
    #[inline(always)]
    pub const fn from_usize(n: usize) -> Option<Self> {
        if n < Self::COUNT {
            Some(unsafe { Self::from_usize_unchecked(n) })
        } else {
            None
        }
    }

    /// Returns the system call number corresponding to this [`SystemCall`].
    #[inline(always)]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
}

/// A size supported by the [`map_memory`] system call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// A 4 KiB page.
    Page4KiB = 1,
    /// A 2 MiB page.
    Page2MiB = 2,
    /// A 1 GiB page.
    Page1GiB = 3,
}

/// Describes a page entry which can be passed to the [`map_memory`] system call.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MappingEntry(pub u64);

impl MappingEntry {
    const READABLE_MASK: u64 = 1 << 0;
    const WRITABLE_MASK: u64 = 1 << 1;
    const EXECUTABLE_MASK: u64 = 1 << 2;

    /// Returns whether the pages are readable.
    #[inline(always)]
    pub const fn readable(self) -> bool {
        self.0 & Self::READABLE_MASK != 0
    }

    /// Returns whether the pages are writable.
    #[inline(always)]
    pub const fn writable(self) -> bool {
        self.0 & Self::WRITABLE_MASK != 0
    }

    /// Returns whether the pages are executable.
    #[inline(always)]
    pub const fn executable(self) -> bool {
        self.0 & Self::EXECUTABLE_MASK != 0
    }

    /// Creates a new [`MappingEntry`] with the additional "readable" flag set to `yes`.
    #[inline(always)]
    pub const fn with_readable(mut self, yes: bool) -> Self {
        if yes {
            self.0 |= Self::READABLE_MASK;
        } else {
            self.0 &= !Self::READABLE_MASK;
        }

        self
    }

    /// Creates a new [`MappingEntry`] with the additional "writable" flag set to `yes`.
    #[inline(always)]
    pub const fn with_writable(mut self, yes: bool) -> Self {
        if yes {
            self.0 |= Self::WRITABLE_MASK;
        } else {
            self.0 &= !Self::WRITABLE_MASK;
        }

        self
    }

    /// Creates a new [`MappingEntry`] with the additional "executable" flag set to `yes`.
    #[inline(always)]
    pub const fn with_executable(mut self, yes: bool) -> Self {
        if yes {
            self.0 |= Self::EXECUTABLE_MASK;
        } else {
            self.0 &= !Self::EXECUTABLE_MASK;
        }

        self
    }

    /// Returns the number of pages to map with this entry.
    #[inline(always)]
    pub const fn count(self) -> u64 {
        self.0 >> 52
    }

    /// Creates a new [`MappingEntry`] with the additional "count" field set to `count`.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if `count` is greater than `0xFFF`. In release builds,
    /// it will simply be truncated.
    #[inline(always)]
    pub const fn with_count(mut self, count: u64) -> Self {
        debug_assert!(count <= 0xFFF);

        self.0 &= !(0xFFF << 52);
        self.0 |= count << 52;

        self
    }

    /// Returns the size of the pages to map with this entry.
    ///
    /// When `None`, the pages should be unmapped instead.
    #[inline(always)]
    pub const fn size(self) -> Option<PageSize> {
        match (self.0 >> 3) & 0b11 {
            0 => None,
            1 => Some(PageSize::Page4KiB),
            2 => Some(PageSize::Page2MiB),
            3 => Some(PageSize::Page1GiB),
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Creates a new [`MappingEntry`] with the additional "size" field set to `size`.
    #[inline(always)]
    pub const fn with_size(mut self, size: Option<PageSize>) -> Self {
        let size = if let Some(size) = size {
            size as u64
        } else {
            0
        };

        self.0 &= !(0b11 << 3);
        self.0 |= size << 3;
        self
    }

    /// Creates a new [`MappingEntry`] with the additional "address" field set to `addr`.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if `addr` is not page-aligned. In release builds, it
    /// will be truncated.
    #[inline(always)]
    pub const fn with_address(mut self, addr: u64) -> Self {
        debug_assert!(
            addr & 0xFFF00000_00000FFF == 0,
            "address must be page-aligned"
        );

        self.0 &= 0xFFF00000_00000FFF;
        self.0 |= addr;

        self
    }

    /// Returns the address of the page entry.
    #[inline(always)]
    pub const fn address(self) -> u64 {
        self.0 & 0x000FFFFF_FFFFF000
    }
}
