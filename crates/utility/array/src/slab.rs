use crate::Vec;

/// An index map.
pub struct Slab<T, const N: usize> {
    data: Vec<Option<T>, N>,
    first_free: usize,
}

impl<T, const N: usize> Slab<T, N> {
    /// Creates a new empty [`IndexMap<T, N>`].
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            first_free: 0,
        }
    }

    /// Returns whether the [`Slab<T, N>`] is full.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.data.is_full() && self.first_free == self.data.len()
    }

    /// Pushes a new value into the [`Slab<T, N>`].
    ///
    /// # Safety
    ///
    /// The [`Slab<T, N>`] must not be full.
    pub unsafe fn insert_unchecked(&mut self, value: T) -> usize {
        debug_assert!(!self.is_full());

        if self.first_free == self.data.len() {
            let index = self.data.len();
            self.data.push_unchecked(Some(value));

            // There was no first free element, and we just added one. There's still not first
            // free element.
            self.first_free = self.data.len();

            index
        } else {
            let index = self.first_free;
            *self.data.get_unchecked_mut(self.first_free) = Some(value);
            self.first_free += 1;

            // We have to find a new first free element.
            while let Some(slot) = self.data.get(self.first_free) {
                if slot.is_none() {
                    // We found a free slot!
                    break;
                }

                self.first_free += 1;
            }

            index
        }
    }

    /// Attempts to push a new value into the [`Slab<T, N>`].
    ///
    /// # Errors
    ///
    /// This function fails if the [`Slab<T, N>`] is full.
    #[inline]
    pub fn insert(&mut self, value: T) -> Result<usize, T> {
        if self.is_full() {
            Err(value)
        } else {
            Ok(unsafe { self.insert_unchecked(value) })
        }
    }

    /// Removes the value at the given index from the [`Slab<T, N>`].
    ///
    /// # Safety
    ///
    /// The index must be valid.
    #[inline]
    pub fn remove_unchecked(&mut self, index: usize) -> T {
        debug_assert!(self.data[index].is_some());

        // We're removing an element that's before the current first free element.
        if index < self.first_free {
            self.first_free = index;
        }

        unsafe { self.data.get_unchecked_mut(index).take().unwrap_unchecked() }
    }

    /// Removes the value at the given index from the [`Slab<T, N>`].
    ///
    /// # Errors
    ///
    /// This function fails if the index is invalid.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        // We're removing an element that's before the current first free element.
        //
        // Note that this works even if `index` is out of bounds or invalid. Because `first_free`
        // is always less than or equal to `data.len()`, we'll never set `first_free` to an
        // invalid index. Moreover, if `index` points to a empty slot, then we know that
        // `first_free` will be less or equal to `index`, thus maintaining the invariant.
        if index < self.first_free {
            self.first_free = index;
        }

        self.data.get_mut(index)?.take()
    }
}
