use core::mem::MaybeUninit;
use core::ops::Deref;

use nd_x86_64::{PhysAddr, VirtAddr};

/// Stores information about the kernel, relevant to the `x86_64` architecture.
///
/// This type is normally accessed through the [`SysInfoTok`] token type.
pub struct SysInfo {
    /// The starting physical address of the kernel in physical memory.
    pub kernel_phys_addr: PhysAddr,
    /// The virtual address of the end of the kernel. This is one byte past the end of the
    /// kernel image in virtual memory.
    pub kernel_virt_end_addr: VirtAddr,
    /// The virtual address of the kernel.
    pub kernel_virt_addr: VirtAddr,
}

impl SysInfo {
    /// Reads the kernel virtual address from the linker script.
    #[inline(always)]
    pub fn read_kernel_virt_addr() -> VirtAddr {
        extern "C" {
            static mut __nd_image_start: u8;
        }

        unsafe { &__nd_image_start as *const _ as usize as VirtAddr }
    }

    /// Reads the kernel end virtual address from the linker script.
    #[inline(always)]
    pub fn read_kernel_virt_end_addr() -> VirtAddr {
        extern "C" {
            static mut __nd_image_end: u8;
        }

        unsafe { &__nd_image_end as *const _ as usize as VirtAddr }
    }
}

/// The global system info object, protected by [`SysInfoTok`].
static mut SYS_INFO: MaybeUninit<SysInfo> = MaybeUninit::uninit();

/// A "token type" proving that the global [`SysInfoTok`] structure has been initialized.
#[derive(Clone, Copy)]
pub struct SysInfoTok(());

impl SysInfoTok {
    /// Creates a new [`SysInfoTok`] instance.
    ///
    /// # Safety
    ///
    /// The [`SysInfoTok::initialize`] function must have been called before this function is
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
    pub unsafe fn initialize(sys_info: SysInfo) -> Self {
        unsafe {
            SYS_INFO.write(sys_info);
            Self::unchecked()
        }
    }
}

impl Deref for SysInfoTok {
    type Target = SysInfo;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { SYS_INFO.assume_init_ref() }
    }
}
