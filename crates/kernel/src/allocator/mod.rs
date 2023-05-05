//! Defines the kernel's memory allocator.

use core::mem::MaybeUninit;

#[cfg(target_arch = "x86_64")]
pub mod paging;

/// The allocator type used throughout the kernel.
#[cfg(target_arch = "x86_64")]
type KernelAllocator = self::paging::PageBasedAllocator;

static mut KERNEL_ALLOCATOR: MaybeUninit<KernelAllocator> = MaybeUninit::uninit();

/// Initializes the kernel's memory allocator.
///
/// # Safety
///
/// This function must be called once!
///
/// On **x86_64**, this function must be called before the kernel's paging system is initialized.
#[inline(always)]
pub unsafe fn initialize_allocator() {
    unsafe { KERNEL_ALLOCATOR.write(KernelAllocator::new()) };
}

/// Returns a reference to the kernel's memory allocator.
///
/// # Safety
///
/// This function may only be called after the kernel's memory allocator has been initialized
/// using the [`initialize_allocator`] function.
#[inline(always)]
pub unsafe fn kernel_allocator() -> &'static KernelAllocator {
    unsafe { KERNEL_ALLOCATOR.assume_init_ref() }
}
