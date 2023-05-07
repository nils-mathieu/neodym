//! Code specific to the `x86_64` CPU architecture.

use core::mem::MaybeUninit;

use nd_x86_64::PhysAddr;
use nd_x86_64::VirtAddr;

mod apic;
mod interrupts;
mod logger;
mod paging;
mod process;
mod tables;

pub use self::apic::*;
pub use self::logger::*;
pub use self::paging::*;
pub use self::process::*;
pub use self::tables::*;

/// Stores information about the kernel, relevant to the `x86_64` architecture.
pub struct KernelInfo {
    /// The starting address of the higher half direct map in the kernel's address space.
    ///
    /// This is also used when mapping to the kernel in processes.
    pub hhdm_offset: VirtAddr,
    /// The number of bytes that the kernel takes, in memory.
    pub kernel_size: usize,
    /// The starting physical address of the kernel in physical memory.
    pub kernel_addr: PhysAddr,
}

static mut KERNEL_INFO: MaybeUninit<KernelInfo> = MaybeUninit::uninit();

/// Initializes the global `KERNEL_INFO` object.
///
/// # Safety
///
/// This function must only be called once!
#[inline(always)]
pub unsafe fn initialize_kernel_info(info: KernelInfo) {
    unsafe { KERNEL_INFO.write(info) };
}

/// Returns a reference to the global `KERNEL_INFO` object.
///
/// # Safety
///
/// This function may only be called *after* the `KERNEL_INFO` object has been initialized using the
/// [`initialize_kernel_info`] function.
#[inline(always)]
pub unsafe fn kernel_info() -> &'static KernelInfo {
    unsafe { KERNEL_INFO.assume_init_ref() }
}

/// Disables interrupts and halts the CPU.
///
/// This function can be called when an unrecoverable error occurs.
pub fn die() -> ! {
    unsafe {
        nd_x86_64::cli();

        loop {
            nd_x86_64::hlt();
        }
    }
}
