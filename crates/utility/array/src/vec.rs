use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// An array-based vector.
pub struct Vec<T, const N: usize> {
    /// The array backing the vector.
    data: [MaybeUninit<T>; N],
    /// The length of the vector.
    len: usize,
}

impl<T, const N: usize> Vec<T, N> {
    const UNINIT_ELEM: MaybeUninit<T> = MaybeUninit::uninit();
    const UNINIT_DATA: [MaybeUninit<T>; N] = [Self::UNINIT_ELEM; N];

    /// Creates a new empty [`Vec<T, N>`].
    pub const fn new() -> Self {
        Self {
            data: Self::UNINIT_DATA,
            len: 0,
        }
    }

    /// Returns the number of elements in the vector.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the vector contains no elements.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether the vector is full.
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Returns a pointer to the array backing this vector.
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    /// Returns a mutable pointer to the array backing this vector.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    /// Attempts to push a new value into the vector.
    ///
    /// This function returns its input in case the vector is full.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        unsafe { self.push_unchecked(value) };
        Ok(())
    }

    /// Pushes a new value into the vector without checking whether the vector is full.
    ///
    /// # Safety
    ///
    /// The vector must not be full.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len < N);

        self.data.get_unchecked_mut(self.len).write(value);
        self.len += 1;
    }

    /// Attempts to remove the last value from the vector.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.pop_unchecked()) }
        }
    }

    /// Removes the last element of the slice, without checking whether the slice is empty.
    ///
    /// # Safety
    ///
    /// The slice must be non-empty.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        self.len -= 1;
        unsafe { self.data.get_unchecked(self.len).assume_init_read() }
    }

    /// Removes the element at `index`.
    ///
    /// This function replaces the removed element with the last element of the vector, thus
    /// avoiding moving all elements after `index`.
    ///
    /// # Safety
    ///
    /// `index` must be in bounds.
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        debug_assert!(index < self.len);

        self.len -= 1;

        let tmp = unsafe { self.data.get_unchecked_mut(index).assume_init_read() };

        if index == self.len {
            // There is no hole to fill, we can just return this element.
            return tmp;
        }

        // We need to fill the hole that we just created.
        unsafe {
            let p = self.data.as_mut_ptr();
            core::ptr::copy_nonoverlapping(p.add(self.len), p.add(index), 1);
        }

        tmp
    }

    /// Attempts to remove the element at `index`.
    ///
    /// This function returns [`None`] if `index` is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            None
        } else {
            Some(unsafe { self.swap_remove_unchecked(index) })
        }
    }
}

impl<T, const N: usize> Deref for Vec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const T, self.len) }
    }
}

impl<T, const N: usize> DerefMut for Vec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T, const N: usize> Default for Vec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> AsRef<[T]> for Vec<T, N> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for Vec<T, N> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone, const N: usize> Clone for Vec<T, N> {
    fn clone(&self) -> Self {
        let mut ret = Self::new();
        for elem in self.iter() {
            // SAFETY:
            //  We know that the vector cannot hold more than `N` elements.
            unsafe { ret.push(elem.clone()).unwrap_unchecked() };
        }
        ret
    }
}

impl<T, const N: usize> Drop for Vec<T, N> {
    fn drop(&mut self) {
        let slice: &mut [T] = self;
        unsafe {
            core::ptr::drop_in_place(slice);
        }
    }
}
