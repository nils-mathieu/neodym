use core::fmt;

use crate::Feature;

/// Requests the Limine bootloader to provide the physical address of *Higher Half Direct Map*.
#[derive(Debug)]
#[repr(transparent)]
pub struct Hhdm;

/// The response to the [`Hhdm`] request.
#[repr(C)]
pub struct HhdmResponse {
    offset: u64,
}

impl HhdmResponse {
    /// Returns the virtual address offset of the begining of the *Higher Half Direct Map*.
    #[inline(always)]
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

impl fmt::Debug for HhdmResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HhdmResponse")
            .field("offset", &format_args!("{:#x}", self.offset))
            .finish()
    }
}

impl Feature for Hhdm {
    type Response = HhdmResponse;
    const MAGIC: [u64; 2] = [0x48dcf1cb8ad2b852, 0x63984e959a98244b];
    const EXPECTED_REVISION: u64 = 0;
    const REVISION: u64 = 0;
}
