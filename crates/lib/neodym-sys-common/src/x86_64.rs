//! Structures and constants specific to x86_64.

/// A system call supported on the x86_64 architecture.
///
/// The disciminant of this enum corresponds to the system call number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum SystemCall {
    Ring0,
    Terminate,
}

impl SystemCall {
    /// The number of defined system calls.
    pub const COUNT: usize = 2;

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

/// An available size for a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PageSize {
    /// A 4 KiB page.
    FourKilobytes = 1,
    /// A 2 MiB page.
    TwoMegabytes = 2,
    /// A 1 GiB page.
    OneGigaBytes = 3,
}
