//! The allocator implementation on systems which use paging.

use core::alloc::{AllocError, Allocator, Layout};
use core::mem::{size_of, MaybeUninit};
use core::ptr::NonNull;

use nd_spin::Mutex;

use super::PAGE_SIZE;
use super::{PageBox, PageList};

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
    /// This function returns the index of the page that contained the allocation, if the page is managed by this node.
    ///
    /// # Safety
    ///
    /// If the block was allocated in one of the pages managed by this node, then it must reference a valid allocation.
    pub unsafe fn deallocate_unchecked(
        &mut self,
        addr: NonNull<u8>,
        slot_idx: usize,
        slot_count: usize,
    ) -> Option<usize> {
        let index = self
            .pages
            .iter_mut()
            .position(move |page| page.includes(addr))?;

        // SAFETY:
        //  This index comes from `position`.
        let page = unsafe { self.pages.get_unchecked_mut(index) };

        // SAFETY:
        //  The caller must make sure that this operation is safe.
        unsafe { page.deallocate_unchecked(slot_idx, slot_count) };

        if page.state == 0 {
            // The page is now empty.
            unsafe { self.pages.swap_remove_unchecked(index) };
        }

        Some(index)
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

/// Allocates a bunch of contiguous slots.
///
/// # Safety
///
/// `slot_count` must not exceed `SLOT_COUNT`.
///
/// `slot_align` must be between `SLOT_SIZE` and `PAGE_SIZE`.
unsafe fn allocate_in_list(
    list: &mut PageList<PageMetaNode>,
    slot_count: usize,
    slot_align: usize,
) -> Result<NonNull<[u8]>, AllocError> {
    if let Some(ok) = list
        .iter_mut()
        .find_map(|node| node.allocate_in_existing(slot_count, slot_align))
    {
        return Ok(ok);
    }

    // We couldn't find a page with enough free slot in the whole list.
    // We need to find a node with a free slot to store a new page.
    let mut meta = unsafe { PageMeta::new()? };

    // SAFETY:
    //  We just created the page, so we know that it's free.
    let result = unsafe { meta.allocate_at_unchecked(0, slot_count) };

    let mut cur = list.cursor();
    while let Some(node) = cur.current_mut() {
        match node.pages.push(meta) {
            Ok(()) => return Ok(result),
            Err(err) => meta = err,
        }

        cur = unsafe { cur.into_next().unwrap_unchecked() };
    }

    // We couldn't find a node with a free slot to store the new page.
    // We have to allocate a new node.
    cur.insert(PageMetaNode::new()).map_err(|_| AllocError)?;

    // SAFETY:
    //  We know that this list is free, since we just allocated it.
    unsafe {
        cur.current_mut()
            .unwrap_unchecked()
            .pages
            .push_unchecked(meta)
    };

    Ok(result)
}

/// Deallocates a block from the provided list.
///
/// # Safety
///
/// The provided address and slot_count must reference an existing block.
unsafe fn deallocate_in_list(
    list: &mut PageList<PageMetaNode>,
    addr: NonNull<u8>,
    slot_count: usize,
) {
    let slot_idx = ((addr.as_ptr() as usize) % PAGE_SIZE) / SLOT_SIZE;

    let mut cur = list.cursor();
    while let Some(node) = cur.current_mut() {
        if unsafe {
            node.deallocate_unchecked(addr, slot_idx, slot_count)
                .is_some()
        } {
            if node.pages.is_empty() {
                cur.remove();
            }

            return;
        }

        cur = unsafe { cur.into_next().unwrap_unchecked() };
    }

    debug_assert!(false, "did not find the block to deallocate");
    unsafe { core::hint::unreachable_unchecked() };
}

/// The inner state of the allocator, when running on **x86_64**.
pub struct PageBasedAllocator {
    head: Mutex<PageList<PageMetaNode>>,
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
            head: Mutex::new(PageList::new()),
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

        unsafe { allocate_in_list(&mut self.head.lock(), slot_count, slot_align) }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let slot_count = (layout.size() + SLOT_SIZE - 1) / SLOT_SIZE;
        unsafe { deallocate_in_list(&mut self.head.lock(), ptr, slot_count) };
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
        let mut iter = lock.iter_mut();
        loop {
            let Some(node) = iter.next() else {
                debug_assert!(false, "did not find the block to grow");
                unsafe { core::hint::unreachable_unchecked() };
            };

            match unsafe { node.grow_unchecked(ptr, slot_idx, slot_count, new_slot_count) } {
                Ok(ok) => return Ok(ok),
                Err(usize::MAX) => (),
                Err(_) => break,
            }
        }

        // We couldn't find the block to grow.
        // Create a new allocation.
        let slot_align = if new_layout.align() < 64 {
            1
        } else {
            new_layout.align() / SLOT_SIZE
        };
        let new = unsafe { allocate_in_list(&mut lock, new_slot_count, slot_align) }?;

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
            deallocate_in_list(&mut lock, ptr, slot_count);
        }

        Ok(new)
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
