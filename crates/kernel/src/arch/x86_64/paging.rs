use core::fmt;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicBool, AtomicUsize};

use nd_array::Vec;
use nd_spin::{Mutex, MutexLock};
use nd_x86_64::{PageTable, PhysAddr, VirtAddr};

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

/// Contains the state of the physical memory allocator.
///
/// This structure may be used to find free physical memory regions, and more generally, map
/// physical memory regions to virtual memory regions when needed.
pub struct PageAllocator {
    /// The usable memory segments.
    l4: &'static mut PageTable,
    /// A list of all usable memory segments.
    segments: Vec<MemorySegment, { Self::MAX_SEGMENT_COUNT }>,
}

/// The global page allocator.
static mut PAGE_ALLOCATOR: MaybeUninit<Mutex<PageAllocator>> = MaybeUninit::uninit();

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
pub unsafe fn lock_page_allocator() -> MutexLock<'static, PageAllocator> {
    #[cfg(debug_assertions)]
    assert!(
        is_initialized(),
        "The page allocator has not been initialized."
    );

    unsafe { &*PAGE_ALLOCATOR.as_ptr() }.lock()
}

impl PageAllocator {
    /// The maximum number of segments that can be managed by the page allocator.
    pub const MAX_SEGMENT_COUNT: usize = 16;
}

/// An iterator over the pages of a memory segment.
struct Pages(MemorySegment);

impl Pages {
    /// Creates a new iterator over the pages of the given memory segment.
    ///
    /// # Panics
    ///
    /// This function panics on debug builds if the provided segment is not aligned to a page
    /// boundary.
    #[inline(always)]
    pub fn new(segment: MemorySegment) -> Self {
        debug_assert_eq!(segment.length & 0xFFF, 0);
        debug_assert_eq!(segment.base & 0xFFF, 0);
        Self(segment)
    }
}

impl Iterator for Pages {
    type Item = PhysAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.length == 0 {
            return None;
        }

        let ret = self.0.base;
        self.0.base += 0x1000;
        self.0.length -= 0x1000;
        Some(ret)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n as u64 * 0x1000 >= self.0.length {
            return None;
        }

        self.0.base += n as u64 * 0x1000;
        self.0.length -= n as u64 * 0x1000;
        self.next()
    }
}

impl ExactSizeIterator for Pages {
    #[inline(always)]
    fn len(&self) -> usize {
        (self.0.length / 0x1000) as usize
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

/// Initializes the paging system of the kernel.
///
/// The interface provided by this module can be used to manipulates the memory management unit of
/// the CPU.
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
/// This function assumes that the current page table is identity-mapped, meaning that the virtual
/// addresses are equal to the physical addresses. This is required because this function will
/// modify the page table, and if it is not identity mapped, it won't be able to access it.
///
/// Note that this function will take ownership of all provided useable memory regions. This means
/// that accessing those regions after this function has been called without first going through
/// memory management functions may result in undefined behavior. Note that the provided iterator
/// may reference data within usable memory. It will be consumed before the memory manager
/// initializes itself.
///
/// Also, after this function has been called, the page tables will be logically owned by the
/// page allocator. Accessing it outside of the module will trigger undefined behavior.
pub unsafe fn initialize_paging(usable: &mut dyn Iterator<Item = MemorySegment>) {
    #[cfg(debug_assertions)]
    assert!(
        !is_initialized(),
        "The page allocator has already been initialized."
    );

    nd_log::trace!("Initializing the page allocator...");

    // SAFETY:
    //  1. The caller must make sure that the page table is identity mapped, ensuring that there is
    //     no conversion to perform between physical and virtual addresses.
    //  2. The caller must make sure that we can take ownership of the page tables.
    let l4 = unsafe { &mut *(nd_x86_64::cr3().addr() as *mut PageTable) };

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

    // Write available segments to that first page.

    // Use the first available page to store available segments.

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
        PAGE_ALLOCATOR.write(Mutex::new(PageAllocator { l4, segments }));
    }

    #[cfg(debug_assertions)]
    PAGE_ALLOCATOR_INITIALIZED.store(true, Release);
}

/// Stores the shared state of a page.
struct PageState {
    /// The virtual address of the page.
    virt: VirtAddr,
    /// The physical address of the page.
    phys: PhysAddr,
    /// The number of references to this page.
    ref_count: AtomicUsize,
}

/// A reference to an allocate page.
pub struct Page {
    state: NonNull<PageState>,
}

unsafe impl Send for Page {}
unsafe impl Sync for Page {}

impl Page {
    #[inline(always)]
    fn state(&self) -> &PageState {
        // SAFETY:
        //  The page state is kept alive by the page allocator as long as the reference count is
        //  not zero. We know that we hold a reference to the page state, we know that the
        //  count is not zero.
        unsafe { self.state.as_ref() }
    }
}

impl Clone for Page {
    fn clone(&self) -> Self {
        // Relaxed ordering is sufficient here. We know that the count is at least 1 (as we hold
        // a reference), ensuring that the page is not dropped while we're cloning it.
        let old_size = self.state().ref_count.fetch_add(1, Relaxed);

        // This might be a data race if the counter overflows and someone detects it and attempts
        // to free the page. This can only happen if the counter goes from `isize::MAX` to
        // `usize::MAX` between the `fetch_add` and this check. Seems unlikely.
        if old_size > isize::MAX as usize {
            // This panic is fine because we're always aborting in case panic. No unwinding can
            // occur here.
            panic!(
                "A page reference count seems to have leaked. (ref_count = {})",
                old_size + 1
            );
        }

        Self { state: self.state }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        // Fetch sub is already atomic, we don't need to syncronize with other threads unless we're
        // going to free the page.
        if self.state().ref_count.fetch_sub(1, Release) == 1 {
            // This subtraction caused the page to be freed. We need to make sure that all
            // modifications to the page are visible to other threads.
            core::sync::atomic::fence(Acquire);
        }
    }
}
