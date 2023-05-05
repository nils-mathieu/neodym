use core::mem::size_of;

use super::PageBox;
use super::PAGE_SIZE;

struct Node<T> {
    next: Option<PageBox<Node<T>>>,
    data: T,
}

impl<T> Node<T> {
    /// Ensures at compile-time that the size of a `Node` is less than or equal to the size of a page.
    const _SIZE_CHECK: () = assert!(size_of::<Self>() <= PAGE_SIZE);
}

/// A linked-list of pages allocated by the global page allocator.
pub struct PageList<T> {
    head: Option<PageBox<Node<T>>>,
}

impl<T> PageList<T> {
    /// Creates a new empty [`PageList`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Returns a cursor over the nodes of this [`PageList`].
    #[inline(always)]
    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor {
            current: &mut self.head,
        }
    }

    /// Returns an iterator over the elements of this [`PageList`].
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.head.as_deref_mut())
    }

    /// Returns an iterator over the elements of this [`PageList`].
    #[inline(always)]
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_deref())
    }
}

impl<'a, T> IntoIterator for &'a PageList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut PageList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A cursor over the nodes of a [`PageList`].
pub struct Cursor<'a, T> {
    current: &'a mut Option<PageBox<Node<T>>>,
}

impl<'a, T> Cursor<'a, T> {
    /// Returns a reference to the current element.
    ///
    /// # Errors
    ///
    /// This function returns [`None`] if the cursor points to the end of the list.
    #[inline(always)]
    pub fn current_mut(&mut self) -> Option<&mut T> {
        self.current.as_mut().map(|node| &mut node.data)
    }

    /// Advances the cursor once.
    ///
    /// If the cursor already points to the end of the list, this function does nothing and `false` is returned.
    pub fn into_next(mut self) -> Option<Self> {
        match self.current {
            Some(node) => {
                self.current = &mut node.next;
                Some(self)
            }
            None => None,
        }
    }

    /// Returns whether this cursor points to the end of the list.
    #[inline(always)]
    pub fn is_end(&self) -> bool {
        self.current.is_none()
    }

    /// Inserts a new element into the list at this cursor position.
    ///
    /// The new element will be inserted *before* the current element.
    ///
    /// # Errors
    ///
    /// This function returns its input if the allocation fails.
    pub fn insert(&mut self, elem: T) -> Result<(), T> {
        let mut node = unsafe {
            PageBox::new(Node {
                next: None,
                data: elem,
            })
            .map_err(|node| node.data)?
        };

        node.next = self.current.take();
        *self.current = Some(node);

        Ok(())
    }

    /// Attempts to remove an element from this list.
    ///
    /// If the cursor points to the end of the list, this function does nothing and returns [`None`].
    pub fn remove(&mut self) -> Option<T> {
        let Node { data, next } = PageBox::into_inner(self.current.take()?);
        *self.current = next;
        Some(data)
    }

    /// Returns an iterator over the elements past and including the current element.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.current.as_deref_mut())
    }

    /// Returns an iterator over the elements past and including the current element.
    #[inline(always)]
    pub fn iter(&self) -> Iter<T> {
        Iter(self.current.as_deref())
    }
}

impl<'a, 'b, T> IntoIterator for &'a mut Cursor<'b, T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a Cursor<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for Cursor<'a, T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IterMut(self.current.as_deref_mut())
    }
}

/// An iterator over the nodes of a [`PageList`].
pub struct IterMut<'a, T>(Option<&'a mut Node<T>>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.take()?;
        self.0 = ret.next.as_deref_mut();
        Some(&mut ret.data)
    }
}

/// An iterator over the nodes of a [`PageList`].
pub struct Iter<'a, T>(Option<&'a Node<T>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.take()?;
        self.0 = ret.next.as_deref();
        Some(&ret.data)
    }
}
