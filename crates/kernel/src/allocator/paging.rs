//! The allocator implementation on systems which use paging.

use core::alloc::{AllocError, Allocator, Layout};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::mem::{size_of, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use nd_spin::Mutex;

/// The size of a single page.
#[cfg(target_arch = "x86_64")]
const PAGE_SIZE: usize = 4096;

/// The size of a slot in the buddy allocator.
const SLOT_SIZE: usize = PAGE_SIZE / usize::BITS as usize;

/// Returns the number of slots in a page.
const SLOT_COUNT: usize = PAGE_SIZE / SLOT_SIZE;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(PAGE_SIZE % SLOT_SIZE == 0);

/// Metadata about a page that has been allocated by the [`KernelAllocator`].
///
/// Each page can store allocation from 64 to 4096 bytes.
struct PageMeta {
    /// The physical frame that stores the allocations.
    page: PageBox<MaybeUninit<[u8; PAGE_SIZE]>>,
    /// The state of the physical frame.
    ///
    /// # Representation
    ///
    /// Each bit of this value represent a single 64-byte block. If the bit is set, the block is
    /// allocated. Otherwise the block is free.
    state: usize,
}

impl PageMeta {
    /// Creates a new [`PageMeta`].
    ///
    /// # Safety
    ///
    /// The global page allocator must've been initialized previously.
    #[inline(always)]
    pub unsafe fn new() -> Result<Self, AllocError> {
        let page = unsafe { PageBox::new_uninit()? };
        Ok(Self { page, state: 0 })
    }

    /// Returns the mask of the slots that are allocated in this page.
    #[inline(always)]
    pub const fn mask(slot_idx: usize, slot_count: usize) -> usize {
        ((1 << slot_count) - 1) << slot_idx
    }

    /// Sets a bunch of slots in this page as "allocated", and returns a pointer to the allocated
    /// bytes.
    ///
    /// # Safety
    ///
    /// The slots must be free.
    #[inline]
    pub unsafe fn allocate_at_unchecked(
        &mut self,
        slot_idx: usize,
        slot_count: usize,
    ) -> NonNull<[u8]> {
        self.state |= Self::mask(slot_idx, slot_count);

        let bytes = core::ptr::slice_from_raw_parts_mut(
            unsafe { self.page.as_mut_ptr().add(slot_idx * SLOT_SIZE) as *mut u8 },
            slot_count * SLOT_SIZE,
        );

        unsafe { NonNull::new_unchecked(bytes) }
    }

    /// Attempt to allocate a bunch of slots in this page.
    pub fn allocate(&mut self, slot_count: usize, slot_align: usize) -> Option<NonNull<[u8]>> {
        // Only check the page on slot alignment.
        let mut slot_idx = 0;

        while slot_idx < SLOT_COUNT {
            // Verify that every slot required to store the allocation is free.
            if self.state & Self::mask(slot_idx, slot_count) == 0 {
                // The slots are free, allocate them.
                return Some(unsafe { self.allocate_at_unchecked(slot_idx, slot_count) });
            }

            slot_idx += slot_align;
        }

        None
    }

    /// Returns whether this page includes the given address.
    pub fn includes(&self, addr: NonNull<u8>) -> bool {
        let start_addr = self.page.as_ptr() as usize;
        let end_addr = start_addr + PAGE_SIZE;
        let addr = addr.as_ptr() as usize;
        start_addr <= addr && addr < end_addr
    }

    /// Deallocates a slot previously allocated in this page.
    ///
    /// # Safety
    ///
    /// The slot must have been allocated in this page, and it must reference a valid allocation.
    pub unsafe fn deallocate_unchecked(&mut self, slot_idx: usize, slot_count: usize) {
        let mask = Self::mask(slot_idx, slot_count);

        // This does not prevent *all* possible errors, but some sanity checks are better than no sanity checks.
        debug_assert!(self.state & mask == mask, "tried to deallocate a free slot");

        self.state &= !Self::mask(slot_idx, slot_count);
    }

    /// Grows an existing allocation.
    ///
    /// # Safety
    ///
    /// This function does not check whether the given slot is allocated.
    ///
    /// # Errors
    ///
    /// This function fails if there is not enough space to grow the allocation.
    pub unsafe fn grow_unchecked(
        &mut self,
        slot_idx: usize,
        slot_count: usize,
        new_slot_count: usize,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_mask = Self::mask(slot_idx, slot_count);

        // This does not prevent *all* possible errors, but some sanity checks are better than no sanity checks.
        debug_assert!(
            self.state & old_mask == old_mask,
            "tried to grow a free slot"
        );

        let added_mask = Self::mask(slot_idx + slot_count, new_slot_count - slot_count);
        if added_mask & self.state != 0 {
            return Err(AllocError);
        }

        Ok(unsafe { self.allocate_at_unchecked(slot_idx, new_slot_count) })
    }
}

/// A node in the linked list.
struct PageMetaNode {
    next: Option<PageBox<PageMetaNode>>,
    pages: nd_array::Vec<PageMeta, { PageMetaNode::MAX_META_COUNT }>,
}

impl PageMetaNode {
    /// The number of [`PageMeta`] that can be stored in a single [`PageMetaNode`].
    pub const MAX_META_COUNT: usize = (PAGE_SIZE - size_of::<usize>() * 2) / size_of::<PageMeta>();

    /// Creates a new [`PageMetaNode`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            next: None,
            pages: nd_array::Vec::new(),
        }
    }

    /// Attempts to allocate a bunch of slots in one of the pages managed by this node.
    ///
    /// This function won't attempt to allocate a new page.
    pub fn allocate_in_existing(
        &mut self,
        slot_count: usize,
        slot_align: usize,
    ) -> Option<NonNull<[u8]>> {
        self.pages
            .iter_mut()
            .find_map(|page| page.allocate(slot_count, slot_align))
    }

    /// Deallocates a slot.
    ///
    /// # Errors
    ///
    /// This function fails if the block was not allocated in any of the pages managed by this node.
    ///
    /// # Safety
    ///
    /// If the block was allocated in one of the pages managed by this node, then it must reference a valid allocation.
    pub unsafe fn deallocate_unchecked(
        &mut self,
        addr: NonNull<u8>,
        slot_idx: usize,
        slot_count: usize,
    ) -> Result<(), ()> {
        let page = self
            .pages
            .iter_mut()
            .find(move |page| page.includes(addr))
            .ok_or(())?;
        unsafe { page.deallocate_unchecked(slot_idx, slot_count) };
        Ok(())
    }

    /// Attemps to grow a block.
    ///
    /// If the block is not allocated in any of the pages managed by this node, then this function returns an error.
    ///
    /// # Safety
    ///
    /// If the block was allocated in one of the pages managed by this node, then it must reference a valid allocation.
    ///
    /// # Errors
    ///
    /// If the error is `usize::MAX`, then the block was not allocated in any of the pages managed by this node.
    ///
    /// Otherwise, the error is the index of the page in which the block was allocated.
    pub unsafe fn grow_unchecked(
        &mut self,
        addr: NonNull<u8>,
        slot_idx: usize,
        slot_count: usize,
        new_slot_count: usize,
    ) -> Result<NonNull<[u8]>, usize> {
        let (page_idx, page) = self
            .pages
            .iter_mut()
            .enumerate()
            .find(move |(_, page)| page.includes(addr))
            .ok_or(usize::MAX)?;

        unsafe {
            page.grow_unchecked(slot_idx, slot_count, new_slot_count)
                .map_err(move |_| page_idx)
        }
    }
}

/// The inner state of the allocator, when running on **x86_64**.
pub struct PageBasedAllocator {
    head: Mutex<Option<PageBox<PageMetaNode>>>,
}

impl PageBasedAllocator {
    /// Creates a new [`State`].
    ///
    /// # Safety
    ///
    /// The global page allocator must've been initialized previously.
    #[inline(always)]
    pub const unsafe fn new() -> Self {
        Self {
            head: Mutex::new(None),
        }
    }
}

unsafe impl Allocator for PageBasedAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() > PAGE_SIZE || layout.align() > PAGE_SIZE {
            return Err(AllocError);
        }

        // The slot alignment of the allocation.
        let slot_align = if layout.align() < 64 {
            1
        } else {
            layout.align() / SLOT_SIZE
        };

        // The number of slots required to store the allocation.
        let slot_count = (layout.size() + SLOT_SIZE - 1) / SLOT_SIZE;

        let mut lock = self.head.lock();
        let mut cur: &mut Option<PageBox<PageMetaNode>> = &mut lock;

        while let Some(node) = cur {
            if let Some(ok) = node.allocate_in_existing(slot_count, slot_align) {
                return Ok(ok);
            }

            cur = &mut node.next;
        }

        // We couldn't find a page with enough free slot in the whole list.
        // We need to find a node with a free slot to store a new page.
        let mut meta = unsafe { PageMeta::new()? };

        // SAFETY:
        //  We just created the page, so we know that it's free.
        let result = unsafe { meta.allocate_at_unchecked(0, slot_count) };

        cur = &mut lock;
        while let Some(node) = cur {
            match node.pages.push(meta) {
                Ok(()) => {
                    // We know that this page is free, since we just allocated it.
                    return Ok(result);
                }
                Err(err) => meta = err,
            }
        }

        // We couldn't find a node with a free slot to store the new page.
        // We have to allocate a new node.
        let new_node = cur.insert(unsafe { PageBox::new(PageMetaNode::new())? });

        // We know that this list is free, since we just allocated it.
        unsafe { new_node.pages.push_unchecked(meta) };

        Ok(result)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let slot_count = (layout.size() + SLOT_SIZE - 1) / SLOT_SIZE;
        let slot_idx = ((ptr.as_ptr() as usize) % PAGE_SIZE) / SLOT_SIZE;

        let mut lock = self.head.lock();
        let mut cur: &mut Option<PageBox<PageMetaNode>> = &mut lock;

        while let Some(node) = cur {
            if unsafe { node.deallocate_unchecked(ptr, slot_idx, slot_count).is_ok() } {
                return;
            }

            cur = &mut node.next;
        }

        debug_assert!(false, "did not find the block to deallocate");
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if new_layout.size() > PAGE_SIZE || new_layout.align() > PAGE_SIZE {
            return Err(AllocError);
        }

        let slot_count = (old_layout.size() + SLOT_SIZE - 1) / SLOT_SIZE;
        let new_slot_count = (new_layout.size() + SLOT_SIZE - 1) / SLOT_SIZE;
        let slot_idx = ((ptr.as_ptr() as usize) % PAGE_SIZE) / SLOT_SIZE;

        let mut lock = self.head.lock();
        let mut cur: &mut Option<PageBox<PageMetaNode>> = &mut lock;

        while let Some(node) = cur {
            match unsafe { node.grow_unchecked(ptr, slot_idx, slot_count, new_slot_count) } {
                Ok(ok) => return Ok(ok),
                Err(usize::MAX) => (),
                Err(page_idx) => {
                    // We could not grow the allocation in-place. We need a new one.

                    // Create a new allocation.
                    let new = self.allocate(new_layout)?;

                    // Copy the old allocation to the new one.
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            ptr.as_ptr(),
                            new.as_ptr() as *mut u8,
                            old_layout.size(),
                        );
                    }

                    // Deallocate the old allocation.
                    unsafe {
                        node.pages
                            .get_unchecked_mut(page_idx)
                            .deallocate_unchecked(slot_idx, slot_count);
                    }

                    return Ok(new);
                }
            }

            cur = &mut node.next;
        }

        // Original block not found.
        debug_assert!(false, "did not find the block to grow");
        unsafe { core::hint::unreachable_unchecked() };
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let new_ptr = unsafe { self.grow(ptr, old_layout, new_layout)? };

        // Zero the new memory.
        unsafe {
            core::ptr::write_bytes(
                (new_ptr.as_ptr() as *mut u8).add(old_layout.size()),
                0x00,
                new_layout.size() - old_layout.size(),
            );
        }

        Ok(new_ptr)
    }
}

/// A memory page allocated by the global page allocator.
pub struct PageBox<T: ?Sized> {
    page: NonNull<T>,
    _marker: PhantomData<T>,
}

unsafe impl<T: ?Sized + Send> Send for PageBox<T> {}
unsafe impl<T: ?Sized + Sync> Sync for PageBox<T> {}

impl<T> PageBox<T> {
    const _SIZE_CHECK: () = assert!(size_of::<T>() <= PAGE_SIZE);

    /// Allocates a new [`PageBox`] using the global page allocator.
    ///
    /// # Safety
    ///
    /// The global page allocator must have been initialized.
    ///
    /// # Errors
    ///
    /// This function fails if the system is out of physical memory.
    #[inline]
    pub unsafe fn new(value: T) -> Result<Self, AllocError> {
        let page = create_box()?.cast::<T>();

        unsafe { page.as_ptr().write(value) };

        Ok(Self {
            page,
            _marker: PhantomData,
        })
    }
}

impl<T> PageBox<MaybeUninit<T>> {
    /// Creates a new [`PageBox`] without initializing it.
    pub unsafe fn new_uninit() -> Result<Self, AllocError> {
        let page = create_box()?.cast::<MaybeUninit<T>>();

        Ok(Self {
            page,
            _marker: PhantomData,
        })
    }
}

/// Attempts to allocate a new page using the global allocator.
fn create_box() -> Result<NonNull<u8>, AllocError> {
    #[cfg(target_arch = "x86_64")]
    {
        let allocator = unsafe { crate::arch::x86_64::page_allocator() };

        // SAFETY:
        //  If the `PageBox` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        let addr = allocator.allocate().ok_or(AllocError)?;

        let virt_addr = allocator.physical_to_virtual(addr) as *mut u8;

        unsafe { Ok(NonNull::new_unchecked(virt_addr)) }
    }
}

impl<T: ?Sized> PageBox<T> {
    /// Leaks this [`PageBox`].
    #[inline(always)]
    pub fn leak(this: Self) -> &'static mut T {
        let mut this = ManuallyDrop::new(this);
        unsafe { this.page.as_mut() }
    }
}

impl<T: ?Sized> Deref for PageBox<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.page.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for PageBox<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.page.as_mut() }
    }
}

impl<T: ?Sized> Drop for PageBox<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.page.as_ptr()) };

        #[cfg(target_arch = "x86_64")]
        unsafe {
            // SAFETY:
            //  If the `PageBox` could be created, we know that the page allocator has been
            //  initialized. This means that we can safely call `page_allocator()`.
            let page_allocator = crate::arch::x86_64::page_allocator();

            let phys_addr =
                page_allocator.virtual_to_physical(self.page.as_ptr() as *const () as usize as u64);
            crate::arch::x86_64::page_allocator().deallocate(phys_addr);
        }
    }
}
