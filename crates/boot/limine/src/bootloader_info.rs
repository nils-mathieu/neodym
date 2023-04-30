use core::fmt;

use crate::Feature;

/// Requests some information about the bootloader responding to Limine requests.
#[derive(Debug)]
#[repr(transparent)]
pub struct BootloaderInfo;

/// The response to the [`BootloaderInfo`] request.
pub struct BootloaderInfoResponse {
    name: *const i8,
    version: *const i8,
}

unsafe impl Send for BootloaderInfoResponse {}
unsafe impl Sync for BootloaderInfoResponse {}

impl BootloaderInfoResponse {
    /// Returns the name of the bootloader.
    #[inline(always)]
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str(self.name) }
    }

    /// Returns the version of the bootloader.
    #[inline(always)]
    pub fn version(&self) -> &str {
        unsafe { cstr_to_str(self.version) }
    }
}

impl fmt::Debug for BootloaderInfoResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootloaderInfoResponse")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl Feature for BootloaderInfo {
    const MAGIC: [u64; 2] = [0xf55038d8e2a1202f, 0x279426fcf5f59740];
    const REVISION: u64 = 0;
    const EXPECTED_REVISION: u64 = 0;
    type Response = BootloaderInfoResponse;
}

/// Converts a C-like string into a regular Rust string.
///
/// # Safety
///
/// `s` must be a null-terminated string borrowed for the lifetime `'a`. It must be valid UTF-8.
unsafe fn cstr_to_str<'a>(s: *const i8) -> &'a str {
    // This should be provided by compiler builtins.
    extern "C" {
        fn strlen(s: *const i8) -> usize;
    }

    unsafe {
        let len = strlen(s);

        // SAFETY:
        //  The slice.
        let slice = core::slice::from_raw_parts(s as *const u8, len);

        core::str::from_utf8_unchecked(slice)
    }
}
