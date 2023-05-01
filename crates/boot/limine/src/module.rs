use bitflags::bitflags;

use core::ffi::CStr;
use core::fmt;
use core::mem::MaybeUninit;

use crate::Feature;

bitflags! {
    /// Some flags which are configure an [`InternalModule`].
    #[derive(Debug, Clone, Copy)]
    pub struct InternalModuleFlags: u64 {
        /// Indicate that kernel loading should fail if the module is not present.
        const REQUIRED = 1 << 0;
    }
}

/// Describes an internal module.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InternalModule {
    path: *const i8,
    cmdline: *const i8,
    flags: InternalModuleFlags,
}

impl InternalModule {
    /// Creates a new [`InternalModule`] instance.
    ///
    /// # Arguments
    ///
    /// - `path`: The path to the module to load. This path is *relative* to the location of the
    /// kernel.
    /// - `cmdline`: The command line for the given module.
    /// - `flags`: Some additional flags passed to Limine.
    #[inline(always)]
    pub const fn new(
        path: &'static CStr,
        cmdline: &'static CStr,
        flags: InternalModuleFlags,
    ) -> Self {
        Self {
            path: path.as_ptr(),
            cmdline: cmdline.as_ptr(),
            flags,
        }
    }

    /// Returns the path to the module.
    #[inline(always)]
    pub fn path(&self) -> &'static CStr {
        // SAFETY:
        //  This pointer is always null terminated and valid for the `'static` lifetime.
        unsafe { CStr::from_ptr(self.path) }
    }

    /// Returns the cmdline associated with the module.
    #[inline(always)]
    pub fn cmdline(&self) -> &'static CStr {
        // SAFETY:
        //  This pointer is always null terminated and valid for the `'static` lifetime.
        unsafe { CStr::from_ptr(self.cmdline) }
    }
}

impl fmt::Debug for InternalModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path().to_str().unwrap_or("<invalid UTF-8>");
        let cmdline = self.cmdline().to_str().unwrap_or("<invalid UTF-8>");

        f.debug_struct("InternalModule")
            .field("path", &path)
            .field("cmdline", &cmdline)
            .field("flags", &self.flags)
            .finish()
    }
}

/// Requests the modules loaded by the kernel.
#[repr(C)]
pub struct Module {
    internal_module_count: u64,
    internal_modules: *const *const InternalModule,
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl Module {
    /// Creates a new [`Module`] instance.
    #[inline(always)]
    pub const fn new(modules: &'static [&'static InternalModule]) -> Self {
        let internal_module_count = modules.len() as u64;
        let internal_modules = modules.as_ptr() as *const *const InternalModule;

        Self {
            internal_module_count,
            internal_modules,
        }
    }

    /// Returns the internal modules referenced by the structure.
    #[inline(always)]
    pub fn internal_modules(&self) -> &'static [&'static InternalModule] {
        unsafe {
            core::slice::from_raw_parts(
                self.internal_module_count as *const &'static InternalModule,
                self.internal_module_count as usize,
            )
        }
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("internal_modules", &self.internal_modules())
            .finish()
    }
}

/// The response type to the [`Module`] request.
#[repr(C)]
pub struct ModuleResponse {
    module_count: u64,
    modules: *mut *mut FileResponse,
}

unsafe impl Send for ModuleResponse {}
unsafe impl Sync for ModuleResponse {}

impl ModuleResponse {
    /// Returns a shared slice over the files that were loaded as kernel modules.
    #[inline(always)]
    pub fn modules(&self) -> &[&FileResponse] {
        unsafe {
            core::slice::from_raw_parts(
                self.modules as *const &FileResponse,
                self.module_count as usize,
            )
        }
    }

    /// Returns an exclusive slice over the files that were loaded as kernel modules.
    #[inline(always)]
    pub fn modules_mut(&mut self) -> &mut [&mut FileResponse] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.modules as *mut &mut FileResponse,
                self.module_count as usize,
            )
        }
    }
}

impl fmt::Debug for ModuleResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleResponse")
            .field("modules", &self.modules())
            .finish()
    }
}

/// An UUID.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct Uuid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

/// A response containing a file.
///
/// Because this structure contains a *revision number*, it has to be checked before accessing
/// the internal [`File`] instance.
pub struct FileResponse {
    revision: u64,
    file: MaybeUninit<File>,
}

impl FileResponse {
    /// The revision number expected for a [`FileResponse`] instance to be considered "up to date".
    pub const EXPECTED_REVISION: u64 = 0;

    /// Returns the revision number of the [`FileResponse`].
    ///
    /// If this number is less than [`FileResponse::EXPECTED_REVISION`], then the inner [`File`]
    /// structure shouldn't be accessed.
    #[inline(always)]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the inner [`File`] without checking the revision number.
    ///
    /// # Safety
    ///
    /// The revision number must be at least [`FileResponse::EXPECTED_REVISION`].
    #[inline(always)]
    pub unsafe fn file_unchecked(&self) -> &File {
        unsafe { self.file.assume_init_ref() }
    }

    /// Returns the inner [`File`] without checking the revision number.
    ///
    ///
    /// # Safety
    ///
    /// The revision number must be at least [`FileResponse::EXPECTED_REVISION`].
    #[inline(always)]
    pub unsafe fn file_unchecked_mut(&mut self) -> &mut File {
        unsafe { self.file.assume_init_mut() }
    }

    /// Returns the inner [`File`] structure, checking beforehand whether this [`FileResponse`] has
    /// the correct revision number.
    #[inline]
    pub fn file(&self) -> Option<&File> {
        #[allow(clippy::absurd_extreme_comparisons)]
        if self.revision >= Self::EXPECTED_REVISION {
            Some(unsafe { self.file_unchecked() })
        } else {
            None
        }
    }

    /// Returns the inner [`File`] structure, checking beforehand whether this [`FileResponse`] has
    /// the correct revision number.
    #[inline]
    pub fn file_mut(&mut self) -> Option<&mut File> {
        #[allow(clippy::absurd_extreme_comparisons)]
        if self.revision >= Self::EXPECTED_REVISION {
            Some(unsafe { self.file_unchecked_mut() })
        } else {
            None
        }
    }
}

impl Drop for FileResponse {
    fn drop(&mut self) {
        #[allow(clippy::absurd_extreme_comparisons)]
        if self.revision >= Self::EXPECTED_REVISION {
            unsafe { self.file.assume_init_drop() };
        }
    }
}

impl fmt::Debug for FileResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("FileResponse");
        s.field("revision", &self.revision);

        if let Some(file) = self.file() {
            s.field("file", file);
            s.finish()
        } else {
            s.finish_non_exhaustive()
        }
    }
}

/// A limine file which may be loaded as part of
#[repr(C)]
pub struct File {
    address: *mut u8,
    size: u64,
    path: *mut i8,
    cmdline: *mut i8,
    media_type: u32,
    _unused: u32,
    tftp_ip: u32,
    tftp_port: u32,
    partition_index: u32,
    mbr_disk_id: u32,
    gpt_disk_uuid: Uuid,
    gpt_part_uuid: Uuid,
    part_uuid: Uuid,
}

unsafe impl Send for File {}
unsafe impl Sync for File {}

impl File {
    /// Returns the address backing this file.
    #[inline(always)]
    pub fn address(&self) -> *const u8 {
        self.address
    }

    /// Returns the address backing this file.
    #[inline(always)]
    pub fn address_mut(&mut self) -> *mut u8 {
        self.address
    }

    /// Returns a slice over the memory backing this file.
    #[inline(always)]
    pub fn data(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.address, self.size as usize) }
    }

    /// Returns a slice over the memory backing this file.
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.address, self.size as usize) }
    }

    /// Returns the size of the file.
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size as usize
    }

    /// Returns the path to the file within the volume, with a leading slash.
    #[inline(always)]
    pub fn path(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.path) }
    }

    /// Returns the cmdline associated with the file.
    #[inline(always)]
    pub fn cmdline(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.cmdline) }
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("address", &self.address)
            .field("size", &self.size)
            .field("path", &self.path())
            .field("cmdline", &self.cmdline())
            .finish_non_exhaustive()
    }
}

impl Feature for Module {
    const MAGIC: [u64; 2] = [0x3e7e279702be32af, 0xca1c4f3bd1280cee];
    const REVISION: u64 = 1;
    const EXPECTED_REVISION: u64 = 1;
    type Response = ModuleResponse;
}
