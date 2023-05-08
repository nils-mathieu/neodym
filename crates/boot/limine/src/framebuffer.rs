use core::fmt;

use crate::Feature;

/// Requests the Limine bootloader to provide information about available framebuffers.
#[derive(Debug)]
pub struct FramebufferRequest;

/// The response to the [`FramebufferRequest`].
#[repr(C)]
pub struct FramebufferResponse {
    framebuffer_count: u64,
    framebuffers: *mut *mut Framebuffer,
}

unsafe impl Send for FramebufferResponse {}
unsafe impl Sync for FramebufferResponse {}

impl FramebufferResponse {
    /// Returns a slice over the framebuffers reported by Limine.
    #[inline(always)]
    pub fn framebuffers(&self) -> &[&Framebuffer] {
        unsafe {
            core::slice::from_raw_parts(
                self.framebuffers as *const &Framebuffer,
                self.framebuffer_count as usize,
            )
        }
    }

    /// Returns a slice over the framebuffers reported by Limine.
    #[inline(always)]
    pub fn framebuffers_mut(&mut self) -> &mut [&mut Framebuffer] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.framebuffers as *mut &mut Framebuffer,
                self.framebuffer_count as usize,
            )
        }
    }
}

impl fmt::Debug for FramebufferResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramebufferResponse")
            .field("framebuffers", &self.framebuffers())
            .finish()
    }
}

/// Information about a framebuffer reported by the Limine bootloader.
#[repr(C)]
pub struct Framebuffer {
    address: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u64,
    memory_mode: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    _unused: [u8; 7],
    edid_size: u64,
    edid: *mut u8,
    mode_count: u64,
    modes: *mut *mut VideoMode,
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    /// Returns the address of the video memory owned by this framebuffer.
    #[inline(always)]
    pub fn address_mut(&mut self) -> *mut u8 {
        self.address
    }

    /// Returns the address of the video memory owned by this framebuffer.
    #[inline(always)]
    pub fn address(&self) -> *const u8 {
        self.address
    }

    /// Returns a slice over the video memory owned by this framebuffer.
    #[inline(always)]
    pub fn data(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.address, self.pitch as usize * self.height as usize)
        }
    }

    /// Returns a slice over the video memory owned by this framebuffer.
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.address,
                self.pitch as usize * self.height as usize,
            )
        }
    }

    /// Returns the width of the framebuffer, in pixels.
    #[inline(always)]
    pub fn width(&self) -> u64 {
        self.width
    }

    /// Returns the height of the framebuffer, in pixels.
    #[inline(always)]
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the number of bytes each row of pixels in this framebuffer occupies.
    #[inline(always)]
    pub fn pitch(&self) -> u64 {
        self.pitch
    }

    /// Returns the number of bits per pixel in this framebuffer.
    #[inline(always)]
    pub fn bpp(&self) -> u64 {
        self.bpp
    }

    /// Returns the memory model of this framebuffer.
    #[inline(always)]
    pub fn memory_mode(&self) -> u8 {
        self.memory_mode
    }

    /// Returns the size of the red mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn red_mask_size(&self) -> u8 {
        self.red_mask_size
    }

    /// Returns the shift of the red mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn red_mask_shift(&self) -> u8 {
        self.red_mask_shift
    }

    /// Returns the size of the green mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn green_mask_size(&self) -> u8 {
        self.green_mask_size
    }

    /// Returns the shift of the green mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn green_mask_shift(&self) -> u8 {
        self.green_mask_shift
    }

    /// Returns the size of the blue mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn blue_mask_size(&self) -> u8 {
        self.blue_mask_size
    }

    /// Returns the shift of the blue mask in this framebuffer, in bits.
    #[inline(always)]
    pub fn blue_mask_shift(&self) -> u8 {
        self.blue_mask_shift
    }

    /// Returns the *Extended Display Identification Data*.
    ///
    /// <https://en.wikipedia.org/wiki/Extended_Display_Identification_Data>
    #[inline(always)]
    pub fn edid(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.edid, self.edid_size as usize) }
    }

    /// Returns the list of video modes supported by this framebuffer.
    #[inline(always)]
    pub fn video_modes(&self) -> &[&VideoMode] {
        unsafe {
            core::slice::from_raw_parts(self.modes as *const &VideoMode, self.mode_count as usize)
        }
    }
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framebuffer")
            .field("address", &self.address)
            .field("pitch", &self.pitch)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bpp", &self.bpp)
            .finish_non_exhaustive()
    }
}

/// Information about a video mode supported by a [`Framebuffer`].
#[repr(C)]
#[derive(Debug)]
pub struct VideoMode {
    pitch: u64,
    width: u64,
    height: u64,
    bpp: u64,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
}

impl VideoMode {
    /// Returns the number of bytes each row of pixels in this video mode occupies.
    #[inline(always)]
    pub fn pitch(&self) -> u64 {
        self.pitch
    }

    /// Returns the width of the framebuffer, in this video mode, in pixels.
    #[inline(always)]
    pub fn width(&self) -> u64 {
        self.width
    }

    /// Returns the height of the framebuffer, in this video mode, in pixels.
    #[inline(always)]
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the number of bits per pixel in this video mode.
    #[inline(always)]
    pub fn bpp(&self) -> u64 {
        self.bpp
    }

    /// Returns the memory model of this video mode.
    #[inline(always)]
    pub fn memory_model(&self) -> u8 {
        self.memory_model
    }

    /// Returns the size of the red mask in this video mode, in bits.
    #[inline(always)]
    pub fn red_mask_size(&self) -> u8 {
        self.red_mask_size
    }

    /// Returns the shift of the red mask in this video mode, in bits.
    #[inline(always)]
    pub fn red_mask_shift(&self) -> u8 {
        self.red_mask_shift
    }

    /// Returns the size of the green mask in this video mode, in bits.
    #[inline(always)]
    pub fn green_mask_size(&self) -> u8 {
        self.green_mask_size
    }

    /// Returns the shift of the green mask in this video mode, in bits.
    #[inline(always)]
    pub fn green_mask_shift(&self) -> u8 {
        self.green_mask_shift
    }

    /// Returns the size of the blue mask in this video mode, in bits.
    #[inline(always)]
    pub fn blue_mask_size(&self) -> u8 {
        self.blue_mask_size
    }

    /// Returns the shift of the blue mask in this video mode, in bits.
    #[inline(always)]
    pub fn blue_mask_shift(&self) -> u8 {
        self.blue_mask_shift
    }
}

impl Feature for FramebufferRequest {
    const MAGIC: [u64; 2] = [0x9d5827dcd881dd75, 0xa3148604f6fab11b];
    const EXPECTED_REVISION: u64 = 1;
    const REVISION: u64 = 0;
    type Response = FramebufferResponse;
}
