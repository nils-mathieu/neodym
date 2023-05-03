use core::fmt;
use core::mem::{size_of, MaybeUninit};
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize};

use nd_array::Vec;
use nd_spin::Mutex;
use nd_x86_64::PhysAddr;

/// A memory segment that is useable by the kernel.
#[derive(Debug, Clone, Copy)]
pub struct MemorySegment {
    /// The base address of the segment.
    ///
    /// This is a *physcal* address.
    pub base: PhysAddr,
    /// The size of the segment.
    pub length: u64,
}

/// A node that's part of the free page list.
///
/// This is a linked list of free pages that can be used to allocate pages.
struct FreePageListNode {
    /// The next node in the list.
    next: AtomicPtr<FreePageListNode>,
    /// The pages that are available for allocation.
    pages: Mutex<Vec<PhysAddr, { Self::MAX_PAGES }>>,
}

const _: () = assert!(size_of::<FreePageListNode>() <= 4096);

impl FreePageListNode {
    /// The maximum number of pages that can be stored in a single node.
    ///
    /// The three `usize` fields are used for the `next` pointer, the lock in `Mutex<_>` and the
    /// `len` in `Vec<_>`.
    pub const MAX_PAGES: usize = (4096 - size_of::<usize>() * 3) / size_of::<PhysAddr>();

    /// Creates a new empty node.
    pub const fn new() -> Self {
        Self {
            next: AtomicPtr::new(core::ptr::null_mut()),
            pages: Mutex::new(Vec::new()),
        }
    }
}

/// Contains the state of the physical memory allocator.
///
/// This structure may be used to find free physical memory regions.
pub struct PageAllocator {
    /// A list of all usable memory segments.
    segments: Vec<MemorySegment, { Self::MAX_SEGMENT_COUNT }>,
    /// The next free page available for allocation.
    ///
    /// This the nth page in the `segments` array. When a couple of pages need to be allocated
    /// together, this field is used to easily find the next free page.
    next_free: AtomicUsize,
    /// The list of free pages.
    free_pages: AtomicPtr<FreePageListNode>,
}

impl PageAllocator {
    /// The maximum number of segments that can be managed by the page allocator.
    pub const MAX_SEGMENT_COUNT: usize = 16;

    /// Allocates a new physical page.
    pub fn allocate(&self) -> Option<PhysAddr> {
        // First, attempt to find a page in the free list.
        let mut cur = &self.free_pages;

        while let Some(node) = unsafe { cur.load(Acquire).as_ref() } {
            // The free list isn't empty. We might be able to get a page from there.

            // We use `try_lock` to avoid spinning on the lock. If this node is locked, we can just
            // skip it and try the next one.
            if let Some(page) = node.pages.try_lock().and_then(|mut pages| pages.pop()) {
                return Some(page);
            }

            cur = &node.next;
        }

        // NOTE:
        //  It is possible to get here while some memory is still available in the free list. This
        //  can occur because of the `try_lock` above. This is *very* unlikely to happen. The more
        //  free pages there are, the more likely we are to acquire a lock. If there are few free
        //  pages, then either we're out of memory (or close to it), or we still have a lot of
        //  memory available in the usable segments.

        // The index of the page that will be allocated.
        //
        // Relaxed ordering is sufficient here because we only care about the order of the
        // operations on this specific atomic variable. If another threads attempts to allocate
        // a page, their operation will be ordered with respect to this one, and we don't really
        // care if it happens before or after.
        let mut page_index = self.next_free.fetch_add(1, Relaxed) as u64;

        // This executes in O(n), with n being the number of segments.
        // This is fine, as we don't expect to have more than `MAX_SEGMENT_COUNT` segments. It will
        // usually be 4 to 8 segments.
        for segment in &self.segments {
            let page_count = segment.length / 4096;

            if page_index >= page_count {
                // We font the right segment!
                return Some(segment.base + page_index * 4096);
            }

            page_index -= page_count;
            // not in this segment
        }

        // This races with the `fetch_add` above, but if other threads are able to allocate enough
        // pages to overflow an `usize` by the time we get here, then the system is probably having
        // bigger issues than this.
        //
        // If `next_free` overflows, then used segments will start being allocated again. This is
        // actually pretty bad, but there's not much we can do about it without using a lock.
        //
        // I think locking would actually be fine, but it's so unlikely that this will be an issue
        // that the lock-free implementation is probably worth it.
        self.next_free.store(page_index as usize, Relaxed);

        // We're out of memory :(
        None
    }

    /// Deallocates a physical address.
    ///
    /// # Safety
    ///
    /// The given address must have been allocated by this allocator.
    pub unsafe fn deallocate(&self, addr: PhysAddr) {
        // Find a node in the free list that can contain the new free page.
        let mut cur = &self.free_pages;

        while let Some(node) = unsafe { cur.load(Acquire).as_ref() } {
            // We use `try_lock` to avoid spinning on the lock. If this node is locked, we can just
            // skip it and try the next one.
            if let Some(mut pages) = node.pages.try_lock() {
                if pages.push(addr).is_ok() {
                    return; // success
                }
            }

            cur = &node.next;
        }

        // We got to the end of the free list, and we didn't find a node that can store our
        // new page. We need to allocate a new node. We'll use the deallocated node for this.
        //
        // FIXME: This assumes identity mapping. A conversion to a virtual address is needed here.
        unsafe { (addr as *mut FreePageListNode).write(FreePageListNode::new()) };

        cur.store(addr as *mut _, Release);
    }
}

/// Returns a [`fmt::Debug`] implementation that displays the given number of bytes in a human
/// readable format.
fn human_bytes(bytes: u64) -> impl fmt::Display {
    struct Bytes(u64);

    impl fmt::Display for Bytes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut bytes = self.0;

            let mut write_dec =
                |n: u64, dim: &str| write!(f, "{}.{} {}", n / 1024, ((n % 1024) * 100) / 1024, dim);

            if bytes < 1024 {
                return write!(f, "{} B", bytes);
            }

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "KiB");
            }

            bytes /= 1024;

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "MiB");
            }

            bytes /= 1024;

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "GiB");
            }

            bytes /= 1024;

            // wtf so much memory
            write_dec(bytes, "TiB")
        }
    }

    Bytes(bytes)
}

/// The global page allocator.
static mut PAGE_ALLOCATOR: MaybeUninit<PageAllocator> = MaybeUninit::uninit();

/// Tracks whether the global page allocator has been initialized.
#[cfg(debug_assertions)]
static PAGE_ALLOCATOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Returns whether the global page allocator has been initialized.
#[cfg(debug_assertions)]
#[inline(always)]
fn is_initialized() -> bool {
    PAGE_ALLOCATOR_INITIALIZED.load(Acquire)
}

/// Returns the global page allocator.
///
/// # Safety
///
/// This function must only be called *after* [`initialize_paging`] has been called.
///
/// # Panics
///
/// In debug builds, this function panics if the page allocator has not been initialized.
#[inline(always)]
pub unsafe fn page_allocator() -> &'static PageAllocator {
    #[cfg(debug_assertions)]
    assert!(
        is_initialized(),
        "The page allocator has not been initialized."
    );

    // SAFETY:
    //  The caller must make sure that the page allocator has been initialized.
    unsafe { PAGE_ALLOCATOR.assume_init_ref() }
}

/// Initializes the page allocator.
///
/// # Arguments
///
/// `usable` is an iterator over the usable memory regions. This memory will be used by the
/// page allocator when allocating memory. The segments must be aligned to a page boundary (in base
/// and in length). The segments must be sorted by base address. Segments may not overlap, but they
/// can be adjacent.
///
/// # Safety
///
/// This function expects to be called only once.
///
/// Note that this function will take ownership of all provided useable memory regions. This means
/// that accessing those regions after this function has been called without first going through
/// memory management functions may result in undefined behavior. Note that the provided iterator
/// may reference data within usable memory. It will be consumed before the memory manager
/// initializes itself.
///
/// Also, after this function has been called, the page tables will be logically owned by the
/// page allocator. Accessing it outside of the module will trigger undefined behavior.
pub unsafe fn initialize_page_allocator(usable: &mut dyn Iterator<Item = MemorySegment>) {
    #[cfg(debug_assertions)]
    assert!(
        !is_initialized(),
        "The page allocator has already been initialized."
    );

    nd_log::trace!("Initializing the page allocator...");

    let mut segments = Vec::<MemorySegment, { PageAllocator::MAX_SEGMENT_COUNT }>::new();
    let mut pages = 0;
    for segment in usable.take(segments.capacity()) {
        pages += segment.length / 0x1000;

        if let Some(last) = segments.last_mut() {
            // Attempt to merge the current segment with the last one.
            if last.base + last.length == segment.base {
                last.length += segment.length;
                continue;
            }
        }

        unsafe { segments.push_unchecked(segment) };
    }

    let remaining = usable.count();
    if remaining != 0 {
        nd_log::warn!("Too many usable memory regions, {remaining} have been ignored.");
    }

    nd_log::info!(
        "{} pages of usable memory, in {} contiguous segments, {} in total.",
        pages,
        segments.len(),
        human_bytes(pages * 0x1000)
    );

    // SAFETY:
    //  This function can only be called once, ensuring that we're not:
    //  1. overwriting an existing instance of the page allocator.
    //  2. messing with another thread that would be using the page allocator.
    //
    // After this function has been called, the page allocator may only be accessed through
    // shared references.
    unsafe {
        PAGE_ALLOCATOR.write(PageAllocator {
            segments,
            next_free: AtomicUsize::new(0),
            free_pages: AtomicPtr::new(core::ptr::null_mut()),
        });
    }

    #[cfg(debug_assertions)]
    PAGE_ALLOCATOR_INITIALIZED.store(true, Release);
}

/// A physical frame that have been allocated by the page allocator.
///
/// This type automatically deallocates the frame when it is dropped.
pub struct PhysicalFrame(PhysAddr);

impl PhysicalFrame {
    /// Allocates a new [`PhysicalFrame`] using the global page allocator.
    ///
    /// # Safety
    ///
    /// The global page allocator must have been initialized.
    ///
    /// # Errors
    ///
    /// This function fails if the system is out of physical memory.
    #[inline]
    pub unsafe fn allocate() -> Option<Self> {
        // SAFETY:
        //  If the `PhysicalFrame` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        unsafe { page_allocator().allocate().map(Self) }
    }

    /// Returns the physical address of the frame.
    #[inline(always)]
    pub const fn addr(&self) -> PhysAddr {
        self.0
    }
}

impl Drop for PhysicalFrame {
    #[inline]
    fn drop(&mut self) {
        // SAFETY:
        //  If the `PhysicalFrame` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        unsafe { page_allocator().deallocate(self.0) };
    }
}
