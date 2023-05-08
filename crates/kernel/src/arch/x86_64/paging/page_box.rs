use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::mem::{align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::arch::x86_64::{OutOfPhysicalMemory, PageAllocatorTok};

use super::PAGE_SIZE;

/// A memory page allocated by the global page allocator.
///
/// This type uses RAII to automatically deallocate the page when it is dropped.
pub struct PageBox<T: ?Sized> {
    page: NonNull<T>,

    /// This is used by the dropchecker to understand that we will drop a `T`.
    _marker: PhantomData<T>,

    page_allocator: PageAllocatorTok,
}

unsafe impl<T: ?Sized + Send> Send for PageBox<T> {}
unsafe impl<T: ?Sized + Sync> Sync for PageBox<T> {}

impl<T> PageBox<T> {
    /// Allocates a new [`PageBox`] using the global page allocator.
    ///
    /// # Errors
    ///
    /// This function fails if the system is out of physical memory.
    #[inline]
    #[track_caller]
    pub fn new(value: T, page_allocator: PageAllocatorTok) -> Result<Self, T> {
        // Those asserts will be removed by the compiler if they pass.
        assert!(size_of::<T>() <= PAGE_SIZE);
        assert!(align_of::<T>() <= PAGE_SIZE);

        let page = match unsafe { create_box(page_allocator) } {
            Ok(p) => p.cast::<T>(),
            Err(_) => return Err(value),
        };

        unsafe { page.as_ptr().write(value) };

        Ok(Self {
            page,
            _marker: PhantomData,
            page_allocator,
        })
    }

    /// Returns the value stored in this [`PageBox`].
    #[inline(always)]
    pub fn into_inner(b: Self) -> T {
        let this = ManuallyDrop::new(b);
        let ret = unsafe { core::ptr::read(this.page.as_ptr()) };
        unsafe { destroy_box(this.page_allocator, this.page.cast()) };
        ret
    }
}

/// Attempts to allocate a new page using the global allocator.
///
/// # Safety
///
/// The global allocator must have been initialized.
unsafe fn create_box(page_allocator: PageAllocatorTok) -> Result<NonNull<u8>, OutOfPhysicalMemory> {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY:
        //  If the `PageBox` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        let addr = page_allocator.allocate()?;

        // SAFETY:
        //  We know that the page allocator has provided a valid physical address.
        let virt_addr = (addr + page_allocator.kernel_info().hhdm_offset) as *mut u8;

        unsafe { Ok(NonNull::new_unchecked(virt_addr)) }
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
        unsafe {
            core::ptr::drop_in_place(self.page.as_ptr());
            destroy_box(self.page_allocator, self.page.cast());
        }
    }
}

/// Deallocates a page.
unsafe fn destroy_box(page_allocator: PageAllocatorTok, page: NonNull<u8>) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let phys_addr = page.as_ptr() as usize as nd_x86_64::VirtAddr
            - page_allocator.kernel_info().hhdm_offset;

        page_allocator.deallocate(phys_addr);
    }
}
