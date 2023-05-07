//! Defines the kernel's memory allocator.

use core::alloc::{AllocError, Allocator, Layout};
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::arch::x86_64::PageAllocatorTok;

#[cfg(target_arch = "x86_64")]
pub mod paging;

/// The allocator type used throughout the kernel.
#[cfg(target_arch = "x86_64")]
pub type KernelAllocator = self::paging::PageBasedAllocator;

static mut KERNEL_ALLOCATOR: MaybeUninit<KernelAllocator> = MaybeUninit::uninit();

/// A "token type" that proves the global kernel allocator is initialized.
#[derive(Clone, Copy)]
pub struct KernelAllocatorTok(());

impl KernelAllocatorTok {
    /// Creates a new [`KernelAllocator`] instance.
    ///
    /// # Safety
    ///
    /// The [`KernelAllocator::initialize`] function must have been called before this function is
    #[inline(always)]
    pub const unsafe fn unchecked() -> Self {
        Self(())
    }

    /// Initializes the global kernel allocator, returning a token proving that it has been
    ///
    /// # Safety
    ///
    /// This function must only be called once!
    ///
    /// * On **x86_64**, this function must be called when the global page allocator has been
    ///   initialized.
    #[inline(always)]
    pub unsafe fn initialize() -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            KERNEL_ALLOCATOR.write(KernelAllocator::new(PageAllocatorTok::unchecked()));
        }

        unsafe { Self::unchecked() }
    }
}

impl Deref for KernelAllocatorTok {
    type Target = KernelAllocator;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { KERNEL_ALLOCATOR.assume_init_ref() }
    }
}

unsafe impl Allocator for KernelAllocatorTok {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        KernelAllocator::allocate(self, layout)
    }

    #[inline(always)]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        KernelAllocator::allocate_zeroed(self, layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { KernelAllocator::deallocate(self, ptr, layout) }
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { KernelAllocator::grow(self, ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { KernelAllocator::grow_zeroed(self, ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { KernelAllocator::shrink(self, ptr, old_layout, new_layout) }
    }
}
