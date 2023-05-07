/// A fixed-capacity string.
pub struct String<const N: usize> {
    buffer: crate::Vec<u8, N>,
}

impl<const N: usize> String<N> {
    /// Creates a new empty string.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            buffer: crate::Vec::new(),
        }
    }

    /// Returns the capacity of the string.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns the length of the string.
    ///
    /// This is the number of bytes stored in the string, not the number of characters.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns whether the string is empty (i.e. contains no bytes).
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Attempts to push additional character to this [`String`].
    ///
    /// # Errors
    ///
    /// If input string is too large to fit in the remaining capacity of this [`String`], then
    /// `false` is returned and the [`String`] is left unchanged.
    #[must_use = "this function might fail"]
    pub fn push_str(&mut self, s: &str) -> bool {
        let buf = self.buffer.spare_capacity_mut();

        if buf.len() < s.len() {
            return false;
        }

        unsafe {
            core::ptr::copy_nonoverlapping(s.as_ptr(), buf.as_mut_ptr() as *mut u8, s.len());
            self.buffer.set_len(self.buffer.len() + s.len());
        }

        true
    }
}
