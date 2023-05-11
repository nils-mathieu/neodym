use core::mem::{size_of, MaybeUninit};
use core::ops::Deref;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::{Acquire, Release};

use nd_array::Vec;
use nd_spin::Mutex;
use nd_x86_64::PhysAddr;

use super::{OutOfPhysicalMemory, PageProvider};
use crate::x86_64::SysInfoTok;

/// A node that's part of the free page list.
///
/// This is a linked list of free pages that can be used to allocate pages.
struct FreePageListNode {
    /// The next node in the list.
    next: AtomicPtr<FreePageListNode>,
    /// The pages that are available for allocation.
    pages: Mutex<Vec<PhysAddr, { Self::MAX_PAGES }>>,
}

const _: () = assert!(size_of::<FreePageListNode>() == 4096);

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
///
/// This type is normally accessed through the [`PageAllocatorTok`] token type.
pub struct PageAllocator {
    /// The page provider used to allocate fresh physical pages.
    page_provider: PageProvider,
    /// The list of free pages.
    free_pages: AtomicPtr<FreePageListNode>,

    /// Proves that the global system info structure has been initialized.
    sys_info: SysInfoTok,
}

impl PageAllocator {
    /// Returns a token proving that the global system info structure has been initialized.
    #[inline(always)]
    pub fn sys_info(&self) -> SysInfoTok {
        self.sys_info
    }

    /// Returns the page provider used by this allocator.
    #[inline(always)]
    pub fn page_provider(&self) -> &PageProvider {
        &self.page_provider
    }

    /// Allocates a new physical page.
    ///
    /// The returned physical address is guaranteed to be page-aligned.
    ///
    /// Note that you can return the page to the allocator by calling [`PageAllocator::deallocate`].
    pub fn allocate(&self) -> Result<PhysAddr, OutOfPhysicalMemory> {
        // First, attempt to find a page in the free list.
        let mut cur = &self.free_pages;

        while let Some(node) = unsafe { cur.load(Acquire).as_ref() } {
            // The free list isn't empty. We might be able to get a page from there.

            // We use `try_lock` to avoid spinning on the lock. If this node is locked, we can just
            // skip it and try the next one.
            //
            // FIXME:
            //  once a page is empty (and `pages.pop()` fails), it remains in the linked list.
            //  I'm not sure how to safely remove it from the list without a lock.
            if let Some(page) = node.pages.try_lock().and_then(|mut pages| pages.pop()) {
                return Ok(page);
            }

            cur = &node.next;
        }

        // NOTE:
        //  It is possible to get here while some memory is still available in the free list. This
        //  can occur because of the `try_lock` above. This is *very* unlikely to happen. The more
        //  free pages there are, the more likely we are to acquire a lock. If there are few free
        //  pages, then either we're out of memory (or close to it), or we still have a lot of
        //  memory available in the usable segments.

        self.page_provider.allocate()
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
        let page_node_ptr = addr as *mut FreePageListNode; // Identity mapped.

        unsafe { page_node_ptr.write(FreePageListNode::new()) };

        cur.store(page_node_ptr, Release);
    }
}

/// The global page allocator.
static mut PAGE_ALLOCATOR: MaybeUninit<PageAllocator> = MaybeUninit::uninit();

/// A "token type" proving that the global [`PageAllocator`] has been initialized.
#[derive(Clone, Copy)]
pub struct PageAllocatorTok(());

impl PageAllocatorTok {
    /// Returns an instance of [`PageAllocatorTok`].
    ///
    /// # Safety
    ///
    /// The [`PageAllocatorTok::initialize`] function must've been called previously.
    #[inline(always)]
    pub unsafe fn unchecked() -> Self {
        Self(())
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
    pub unsafe fn initialize(sys_info: SysInfoTok, page_provider: PageProvider) -> Self {
        nd_log::trace!("Initializing the page allocator...");

        // SAFETY:
        //  This function can only be called once, ensuring that we're not:
        //  1. overwriting an existing instance of the page allocator.
        //  2. messing with another thread that would be using the page allocator.
        //
        // After this function has been called, the page allocator may only be accessed through
        // shared references.
        unsafe {
            PAGE_ALLOCATOR.write(PageAllocator {
                page_provider,
                free_pages: AtomicPtr::new(core::ptr::null_mut()),
                sys_info,
            });
            Self::unchecked()
        }
    }
}

impl Deref for PageAllocatorTok {
    type Target = PageAllocator;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { PAGE_ALLOCATOR.assume_init_ref() }
    }
}
