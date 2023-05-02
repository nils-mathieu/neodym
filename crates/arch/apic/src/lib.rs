//! An interface to the APIC interrupt architecture.

#![no_std]

use nd_x86_64::PhysAddr;

/// Reads the `IA32_APIC_BASE` MSR and returns the base address of the XAPIC.
///
/// Note that this function returns a *physical* address.
#[inline]
pub fn get_xapic_base() -> PhysAddr {
    /// The address of the `IA32_APIC_BASE` MSR.
    const IA32_APIC_BASE: u32 = 0x1B;

    // SAFETY:
    // - `IA32_APIC_BASE` is a valid MSR address.
    unsafe { nd_x86_64::rdmsr(IA32_APIC_BASE) & 0xFFFFFF000 }
}
