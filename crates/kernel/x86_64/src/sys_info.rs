use core::mem::MaybeUninit;
use core::ops::Deref;

use nd_x86_64::{PhysAddr, VirtAddr};

/// Stores information about the kernel, relevant to the `x86_64` architecture.
///
/// This type is normally accessed through the [`SysInfoTok`] token type.
pub struct SysInfo {
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
