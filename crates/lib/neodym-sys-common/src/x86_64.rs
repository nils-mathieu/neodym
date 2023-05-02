//! Structures and constants specific to x86_64.

/// A system call supported on the x86_64 architecture.
///
/// The disciminant of this enum corresponds to the system call number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum SystemCall {
    /// The `terminate` system call.
    Terminate,
    /// The `yield` system call.
    Yield,
}

impl SystemCall {
    /// The maximum system call number.
    pub const MAX_SYSCALL_NUMBER: usize = 1;

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
        match n {
            0..=Self::MAX_SYSCALL_NUMBER => Some(unsafe { Self::from_usize_unchecked(n) }),
            _ => None,
        }
    }

    /// Returns the system call number corresponding to this [`SystemCall`].
    #[inline(always)]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
}
