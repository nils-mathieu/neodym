//! Defines the kernel's memory allocator.

use core::alloc::{AllocError, Allocator, Layout};
use core::mem::MaybeUninit;
use core::ptr::NonNull;

#[cfg(target_arch = "x86_64")]
pub mod paging;

/// The allocator type used throughout the kernel.
#[cfg(target_arch = "x86_64")]
type ConcreteKernelAllocator = self::paging::PageBasedAllocator;

static mut KERNEL_ALLOCATOR: MaybeUninit<ConcreteKernelAllocator> = MaybeUninit::uninit();

/// Initializes the kernel's memory allocator.
///
/// # Safety
///
/// This function must be called once!
///
/// On **x86_64**, this function must be called before the kernel's paging system is initialized.
#[inline(always)]
pub unsafe fn initialize_allocator() {
    unsafe { KERNEL_ALLOCATOR.write(ConcreteKernelAllocator::new()) };
}

/// Returns a reference to the kernel's memory allocator.
///
/// # Safety
///
/// This function may only be called after the kernel's memory allocator has been initialized
/// using the [`initialize_allocator`] function.
#[inline(always)]
unsafe fn kernel_allocator() -> &'static ConcreteKernelAllocator {
    unsafe { KERNEL_ALLOCATOR.assume_init_ref() }
}

/// An allocator forwards all calls to the kernel's global memory allocator.
pub struct KernelAllocator(());

impl KernelAllocator {
    /// Creates an new [`KernelAllocator`] instance.
    ///
    /// # Safety
    ///
    /// This function must be called after the kernel's memory allocator has been initialized
    /// through the [`initialize_allocator`] function.
    #[inline(always)]
    pub const unsafe fn new() -> Self {
        Self(())
    }
}

unsafe impl Allocator for KernelAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { kernel_allocator().allocate(layout) }
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { kernel_allocator().allocate_zeroed(layout) }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { kernel_allocator().deallocate(ptr, layout) }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { kernel_allocator().grow(ptr, old_layout, new_layout) }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { kernel_allocator().grow_zeroed(ptr, old_layout, new_layout) }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { kernel_allocator().shrink(ptr, old_layout, new_layout) }
    }
}
