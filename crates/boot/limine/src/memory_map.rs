use core::fmt;

use crate::Feature;

/// Requests the Limine bootloader to provide a map of the available physical memory.
#[derive(Debug)]
#[repr(transparent)]
pub struct MemoryMap;

/// The response of to the [`MemoryMap`] request.
#[repr(C)]
pub struct MemoryMapResponse {
    entry_count: u64,
    entries: *mut *mut MemMapEntry,
}

impl MemoryMapResponse {
    /// Returns the number of entries in the memory map.
    #[inline(always)]
    pub fn entries(&self) -> &[&MemMapEntry] {
        unsafe {
            core::slice::from_raw_parts(
                self.entries as *const &MemMapEntry,
                self.entry_count as usize,
            )
        }
    }
}

impl fmt::Debug for MemoryMapResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryMapResponse")
            .field("entries", &self.entries())
            .finish()
    }
}

unsafe impl Send for MemoryMapResponse {}
unsafe impl Sync for MemoryMapResponse {}

/// Information about a specific memory region.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemMapEntry {
    base: u64,
    length: u64,
    ty: MemMapEntryType,
}

impl MemMapEntry {
    /// Returns the base physical address of the memory region.
    #[inline(always)]
    pub fn base(&self) -> u64 {
        self.base
    }

    /// Returns the length of the memory region.
    #[inline(always)]
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Returns the type of the memory region.
    #[inline(always)]
    pub fn ty(&self) -> MemMapEntryType {
        self.ty
    }
}

impl fmt::Debug for MemMapEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemMapEntry")
            .field("base", &format_args!("{:#x}", self.base))
            .field("length", &format_args!("{:#x}", self.length))
            .field("ty", &self.ty)
            .finish()
    }
}

/// The type of a memory map entry.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemMapEntryType(u64);

impl MemMapEntryType {
    /// The segment is usable.
    pub const USABLE: Self = Self(0);
    /// The segment is reserved.
    pub const RESERVED: Self = Self(1);
    /// The segment belongs to the ACPI reclaimable memory.
    pub const ACPI_RECLAIMABLE: Self = Self(2);
    /// The segment is ACPI NVS memory.
    pub const ACPI_NVS: Self = Self(3);
    /// The segment is bad.
    pub const BAD: Self = Self(4);
    /// The segment is a bootloader reserved segment.
    pub const BOOTLOADER_RECLAIMABLE: Self = Self(5);
    /// The segment contains the kernel or its modules.
    pub const KERNEL_AND_MODULES: Self = Self(6);
    /// The segment contains video memory used by the framebuffer.
    pub const FRAMEBUFFER: Self = Self(7);
}

impl MemMapEntryType {
    /// Returns a string representation of this value.
    pub const fn name(&self) -> &'static str {
        match *self {
            Self::USABLE => "USEABLE",
            Self::RESERVED => "RESERVED",
            Self::ACPI_RECLAIMABLE => "ACPI_RECLAIMABLE",
            Self::ACPI_NVS => "ACPI_NVS",
            Self::BAD => "BAD",
            Self::BOOTLOADER_RECLAIMABLE => "BOOTLOADER_RECLAIMABLE",
            Self::KERNEL_AND_MODULES => "KERNEL_AND_MODULES",
            Self::FRAMEBUFFER => "FRAMEBUFFER",
            _ => "UNKNOWN",
        }
    }
}

impl fmt::Debug for MemMapEntryType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Feature for MemoryMap {
    const MAGIC: [u64; 2] = [0x67cf3d9d378a806f, 0xe304acdfc50c3c62];
    type Response = MemoryMapResponse;
    const EXPECTED_REVISION: u64 = 0;
    const REVISION: u64 = 0;
}
