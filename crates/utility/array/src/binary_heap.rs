use core::mem::{ManuallyDrop, MaybeUninit};

use crate::Vec;

//
// The implementation of this binary heap is largely based on the one from the Rust standard
// library.
//
// https://doc.rust-lang.org/src/alloc/collections/binary_heap/mod.rs.htm
//

/// A priority queue implemented as a binary heap.
pub struct BinaryHeap<T, const N: usize> {
    data: Vec<T, N>,
}

impl<T, const N: usize> BinaryHeap<T, N> {
    /// Creates a new empty [`BinaryHeap<T, N>`].
    #[inline]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Returns the number of elements in the heap.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns whether the heap contains no elements.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T, const N: usize> BinaryHeap<T, N>
where
    T: PartialOrd,
{
    /// Attempts to push a new element in the heap.
    ///
    /// # Errors
    ///
    /// This function fails if the heap is full.
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let index = self.data.len();
        self.data.push(item)?;
        unsafe { self.sift_up(0, index) };
        Ok(())
    }

    /// Removes the greatest element from the heap and returns it.
    pub fn pop(&mut self) -> Option<T> {
        match self.data.len() {
            0 => None,
            1 => Some(unsafe { self.data.pop_unchecked() }),
            _ => {
                // This swaps the first and the last element and returns the first one.
                let result = unsafe { self.data.swap_remove_unchecked(0) };

                // Restore the heap invariant.
                unsafe { self.sift_down_to_bottom(0) };

                Some(result)
            }
        }
    }

    /// Restores the heap invariant by sifting up the element at `pos`.
    ///
    /// The final position of the element is returned.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `pos < self.len()`.
    unsafe fn sift_up(&mut self, start: usize, pos: usize) -> usize {
        debug_assert!(pos < self.len());

        let mut hole = unsafe { Hole::new(&mut self.data, pos) };

        while hole.pos() > start {
            let parent = (hole.pos() - 1) / 2;

            if hole.element() <= unsafe { hole.get(parent) } {
                break;
            }

            unsafe { hole.move_to(parent) };
        }

        hole.pos()
    }

    /// Takes an element at `pos` and moves it all the way down the heap.
    ///
    /// This function should be used when the element is known to be large and should be closer
    /// to the bottom.
    unsafe fn sift_down_to_bottom(&mut self, mut pos: usize) {
        let end = self.len();
        let start = pos;

        let mut hole = unsafe { Hole::new(&mut self.data, pos) };
        let mut child = 2 * hole.pos() + 1;

        while child < end {
            child += unsafe { hole.get(child) <= hole.get(child + 1) } as usize;
            unsafe { hole.move_to(child) };
            child = 2 * hole.pos() + 1;
        }

        if child == end - 1 {
            unsafe { hole.move_to(child) };
        }

        pos = hole.pos();
        drop(hole);

        unsafe { self.sift_up(start, pos) };
    }
}

/// A hole within a slice.
///
/// Normally, a slice `[T]` is contiguous in memory and all of its items are properly initialized.
/// This type represents a hole within such a slice. When dropped, the [`Hole`] restores the
/// original invariant of the slice by moving an item at the hold position.
struct Hole<'a, T> {
    /// The only uninitialized item in the slice is the one at `pos`.
    data: &'a mut [MaybeUninit<T>],
    /// The position of the hole in `data`.
    pos: usize,
    /// The item that will be moved into the hole when it is dropped.
    elem: ManuallyDrop<T>,
}

impl<'a, T> Hole<'a, T> {
    /// Creates a new [`Hole`] instance.
    ///
    /// # Safety
    ///
    /// `pos` must be in bounds of `data`.
    pub unsafe fn new(data: &'a mut [T], pos: usize) -> Self {
        unsafe {
            let data = &mut *(data as *mut [T] as *mut [MaybeUninit<T>]);
            let elem = ManuallyDrop::new(data.get_unchecked_mut(pos).assume_init_read());
            Self { data, pos, elem }
        }
    }

    /// Returns the position of the hole.
    #[inline(always)]
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns a shared reference to the backup element.
    #[inline(always)]
    pub fn element(&self) -> &T {
        &self.elem
    }

    /// Gets an element of the inner data slice.
    ///
    /// # Safety
    ///
    /// `index` must be less than the length of the original slice and must not equal the position
    /// of the hole.
    pub unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());

        unsafe { self.data.get_unchecked(index).assume_init_ref() }
    }

    /// Moves the hole to a new position.
    ///
    /// The old hole will be filled by the new one.
    ///
    /// # Safety
    ///
    /// `new_pos` must be a valid index within the original slice and must not equal the position
    /// of the old hole.
    pub unsafe fn move_to(&mut self, new_pos: usize) {
        debug_assert!(new_pos < self.data.len());
        debug_assert!(new_pos != self.pos);

        unsafe {
            let p = self.data.as_mut_ptr();
            let src = p.add(new_pos);
            let dst = p.add(self.pos);

            core::ptr::copy_nonoverlapping(src, dst, 1);
        }

        self.pos = new_pos;
    }
}

impl<'a, T> Drop for Hole<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.data
                .get_unchecked_mut(self.pos)
                .write(ManuallyDrop::take(&mut self.elem));
        }
    }
}
