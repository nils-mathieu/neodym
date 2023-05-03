use core::mem::MaybeUninit;

#[cfg(debug_assertions)]
use core::sync::atomic::AtomicBool;
#[cfg(debug_assertions)]
use core::sync::atomic::Ordering::{Acquire, Release};

use nd_x86_64::{PageTable, PhysAddr};

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
/// physical memory regions to virtual memory regions.
pub struct PageAllocator {
    /// The usable memory segments.
    l4: &'static mut PageTable,
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
#[inline(always)]
pub unsafe fn page_allocator() -> &'static PageAllocator {
    #[cfg(debug_assertions)]
    assert!(
        is_initialized(),
        "The page allocator has not been initialized."
    );

    unsafe { &*PAGE_ALLOCATOR.as_ptr() }
}

/// Initializes the paging system of the kernel.
///
/// The interface provided by this module can be used to manipulates the memory management unit of
/// the CPU.
///
/// # Arguments
///
/// * `usable`: An iterator over the usable memory regions. This memory will be used by the
/// page allocator when allocating memory.
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

    let mut count = 0;

    for segment in usable {
        count += segment.length;
    }

    nd_log::trace!("{} bytes ({} Mo) of usable memory.", count, count >> 20);

    // SAFETY:
    //  1. The caller must make sure that the page table is identity mapped, ensuring that there is
    //     no conversion to perform between physical and virtual addresses.
    //  2. The caller must make sure that we can take ownership of the page tables.
    let l4 = unsafe { &mut *(nd_x86_64::cr3().addr() as *mut PageTable) };

    // SAFETY:
    //  This function can only be called once, ensuring that we're not:
    //  1. overwriting an existing instance of the page allocator.
    //  2. messing with another thread that would be using the page allocator.
    //
    // After this function has been called, the page allocator may only be accessed through
    // shared references.
    unsafe {
        PAGE_ALLOCATOR.write(PageAllocator { l4 });
    }

    #[cfg(debug_assertions)]
    PAGE_ALLOCATOR_INITIALIZED.store(true, Release);
}
