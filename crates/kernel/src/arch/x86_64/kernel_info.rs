use core::mem::MaybeUninit;
use core::ops::Deref;

use nd_x86_64::{PhysAddr, VirtAddr};

/// Stores information about the kernel, relevant to the `x86_64` architecture.
pub struct KernelInfo {
    /// The starting address of the higher half direct map in the kernel's address space.
    ///
    /// This is also used when mapping to the kernel in processes.
    pub hhdm_offset: VirtAddr,
    /// The number of bytes that the kernel takes, in memory.
    pub kernel_size: usize,
    /// The starting physical address of the kernel in physical memory.
    pub kernel_phys_addr: PhysAddr,
    /// The virtual address of the kernel.
    pub kernel_virt_addr: VirtAddr,
}

static mut KERNEL_INFO: MaybeUninit<KernelInfo> = MaybeUninit::uninit();

/// A "token type" proving that the global [`KernelInfo`] structure has been initialized.
#[derive(Clone, Copy)]
pub struct KernelInfoTok(());

impl KernelInfoTok {
    /// Creates a new [`KernelInfoTok`] instance.
    ///
    /// # Safety
    ///
    /// The [`KernelInfoTok::initialize`] function must have been called before this function is
    /// called.
    #[inline(always)]
    pub unsafe fn unchecked() -> Self {
        Self(())
    }

    /// Initializes the globa kernel info object, returning a token proving that is has been
    /// initialized.
    ///
    /// # Safety
    ///
    /// This function must only be called once!
    #[inline(always)]
    pub unsafe fn initialize(kernel_info: KernelInfo) -> Self {
        unsafe {
            KERNEL_INFO.write(kernel_info);
            Self::unchecked()
        }
    }
}

impl Deref for KernelInfoTok {
    type Target = KernelInfo;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { KERNEL_INFO.assume_init_ref() }
    }
}
