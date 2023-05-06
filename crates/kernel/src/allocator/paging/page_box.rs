use core::alloc::AllocError;
use core::marker::PhantomData;
use core::mem::size_of;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::arch::x86_64::OutOfPhysicalMemory;

use super::PAGE_SIZE;

/// A memory page allocated by the global page allocator.
pub struct PageBox<T: ?Sized> {
    page: NonNull<T>,

    /// This is used by the dropchecker to understand that we will drop a `T`.
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
    pub unsafe fn new(value: T) -> Result<Self, T> {
        let page = match unsafe { create_box() } {
            Ok(p) => p.cast::<T>(),
            Err(_) => return Err(value),
        };

        unsafe { page.as_ptr().write(value) };

        Ok(Self {
            page,
            _marker: PhantomData,
        })
    }

    /// Allocates a new [`PageBox`], initializing it with zeros.
    ///
    /// # Safety
    ///
    /// The global page allocator must have been initialized.
    ///
    /// The all-zeros bit pattern must be valid for type type `T`.
    ///
    /// # Errors
    ///
    /// This function fails if the system is out of physical memory.
    pub unsafe fn zeroed() -> Result<Self, OutOfPhysicalMemory> {
        let page = unsafe { create_box()? }.cast::<T>();

        unsafe {
            core::ptr::write_bytes(page.as_ptr(), 0, 1);
        }

        Ok(Self {
            page,
            _marker: PhantomData,
        })
    }

    /// Returns the value stored in this [`PageBox`].
    pub fn into_inner(b: Self) -> T {
        let this = ManuallyDrop::new(b);
        let ret = unsafe { core::ptr::read(this.page.as_ptr()) };
        unsafe { destroy_box(this.page.cast()) };
        ret
    }
}

impl<T> PageBox<MaybeUninit<T>> {
    /// Creates a new [`PageBox`] without initializing it.
    pub unsafe fn new_uninit() -> Result<Self, AllocError> {
        let page = unsafe { create_box()? }.cast::<MaybeUninit<T>>();

        Ok(Self {
            page,
            _marker: PhantomData,
        })
    }
}

/// Attempts to allocate a new page using the global allocator.
///
/// # Safety
///
/// The global allocator must have been initialized.
unsafe fn create_box() -> Result<NonNull<u8>, OutOfPhysicalMemory> {
    #[cfg(target_arch = "x86_64")]
    {
        let allocator = unsafe { crate::arch::x86_64::page_allocator() };

        // SAFETY:
        //  If the `PageBox` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        let addr = allocator.allocate()?;

        // SAFETY:
        //  We know that the page allocator has provided a valid physical address.
        let virt_addr = allocator.physical_to_virtual(addr) as *mut u8;

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
            destroy_box(self.page.cast());
        }
    }
}

/// Deallocates a page.
unsafe fn destroy_box(page: NonNull<u8>) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // SAFETY:
        //  If the `PageBox` could be created, we know that the page allocator has been
        //  initialized. This means that we can safely call `page_allocator()`.
        let page_allocator = crate::arch::x86_64::page_allocator();

        // SAFETY:
        //  We know that we kept a valid virtual address to the page, so we can safely convert it
        //  back to a physical address.
        let phys_addr = page_allocator.virtual_to_physical(page.as_ptr() as usize as u64);

        crate::arch::x86_64::page_allocator().deallocate(phys_addr);
    }
}
