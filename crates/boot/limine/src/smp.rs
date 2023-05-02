use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

use bitflags::bitflags;

use crate::Feature;

bitflags! {
    /// Some flags which may be passed to an [`SmpRequest`].
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct SmpRequestFlags: u64 {
        /// Enables the X2APIC, if possible.
        #[cfg(target_arch = "x86_64")]
        const X2APIC = 1 << 0;
    }
}

bitflags! {
    /// Some flags received in an [`SmpResponse`].
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    #[cfg(target_arch = "x86_64")]
    pub struct SmpResponseFlags: u32 {
        /// Whether the X2APIC could be enabled.
        const X2APIC = 1 << 0;
    }
}

/// Requests the Limine bootloader to gather and provide information about the other processors
/// available on the current machine.
///
/// The presence of this request will also prompt the bootloader to boostrap the other processors.
/// This will not be done if the request is not present.
#[repr(C)]
#[derive(Debug)]
pub struct Smp {
    /// Some flags passed to the bootloader.
    pub flags: SmpRequestFlags,
}

/// The response to the [`Smp`] request.
#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub struct SmpResponse {
    flags: u32,
    bsp_lapic_id: u32,
    cpu_count: u64,
    cpus: *const *const SmpInfo,
}

unsafe impl Send for SmpResponse {}
unsafe impl Sync for SmpResponse {}

#[cfg(target_arch = "x86_64")]
impl SmpResponse {
    /// Some flags provided by the bootloader.
    #[inline(always)]
    pub fn flags(&self) -> SmpResponseFlags {
        SmpResponseFlags::from_bits_retain(self.flags)
    }

    /// Returns the ID of the local APIC attached to the bootstrap CPU.
    #[inline(always)]
    pub fn bootstrap_cpu_lapic_id(&self) -> u32 {
        self.bsp_lapic_id
    }

    /// Returns information about all available CPUs.
    #[inline(always)]
    pub fn cpus(&self) -> &[&SmpInfo] {
        unsafe {
            core::slice::from_raw_parts(self.cpus as *const &SmpInfo, self.cpu_count as usize)
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl fmt::Debug for SmpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmpResponse")
            .field("flags", &self.flags())
            .field("boostrap_cpu_lapic_id", &self.bsp_lapic_id)
            .field("cpus", &self.cpus())
            .finish()
    }
}

impl Feature for Smp {
    const MAGIC: [u64; 2] = [0x95a67b819a1b857e, 0xa0b61b723b6a73e0];
    const REVISION: u64 = 0;
    const EXPECTED_REVISION: u64 = 0;
    type Response = SmpResponse;
}

/// Provides information about a CPU core.
#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct SmpInfo {
    processor_id: u32,
    lapic_id: u32,
    _reserved: u64,
    goto_address: AtomicU64,
    extra_argument: AtomicU64,
}

#[cfg(target_arch = "x86_64")]
impl SmpInfo {
    /// Returns the ID of the CPU core.
    #[inline(always)]
    pub fn processor_id(&self) -> u32 {
        self.processor_id
    }

    /// Returns the ID of the local APIC of the CPU core.
    #[inline(always)]
    pub fn lapic_id(&self) -> u32 {
        self.lapic_id
    }

    /// Returns a user-defined argument.
    ///
    /// This value can be written to send information to other CPUs.
    #[inline(always)]
    pub fn extra_argument(&self) -> &AtomicU64 {
        &self.extra_argument
    }

    /// Unparks the CPU core and makes it jump to the provided function.
    #[inline]
    pub fn unpark(&self, to: extern "C" fn(&SmpInfo)) {
        let addr = to as usize as u64;
        self.goto_address.store(addr, Ordering::Relaxed);
    }
}

#[cfg(target_arch = "x86_64")]
impl fmt::Debug for SmpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmpInfo")
            .field("processor_id", &self.processor_id)
            .field("lapic_id", &self.lapic_id)
            .finish_non_exhaustive()
    }
}
