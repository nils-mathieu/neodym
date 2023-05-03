use core::fmt;

use crate::Feature;

/// Requests the Limine bootloader to provide the physical address of the kernel.
#[derive(Debug)]
#[repr(transparent)]
pub struct KernelAddress;

/// The response to the [`KernelAddress`] request.
#[repr(C)]
pub struct KernelAddressResponse {
    physical_base: u64,
    virtual_base: u64,
}

impl KernelAddressResponse {
    /// Returns the physical base address of the kernel.
    #[inline(always)]
    pub fn physical_base(&self) -> u64 {
        self.physical_base
    }

    /// Returns the virtual base address of the kernel.
    #[inline(always)]
    pub fn virtual_base(&self) -> u64 {
        self.virtual_base
    }
}

impl fmt::Debug for KernelAddressResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KernelAddressResponse")
            .field("physical_base", &format_args!("{:#x}", self.physical_base))
            .field("virtual_base", &format_args!("{:#x}", self.virtual_base))
            .finish()
    }
}

impl Feature for KernelAddress {
    type Response = KernelAddressResponse;
    const MAGIC: [u64; 2] = [0x71ba76863cc55f63, 0xb2644a48c516a487];
    const EXPECTED_REVISION: u64 = 0;
    const REVISION: u64 = 0;
}
