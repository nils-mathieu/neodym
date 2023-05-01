//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

/// The type of file to be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// The process will start by jumping at the begining of the file.
    Bin = 1,
    /// An ELF file should be loaded.
    ///
    /// In that case, only the ELF header will be parsed and the kernel will start the process
    /// on the entry point specified there.
    Elf,
}

impl FileType {
    /// Returns the [`FileType`] instance associated with the specified stringy type.
    ///
    /// If `s` is not associated with any [`FileType`], [`None`] is returned.
    pub const fn from_bytes(s: &[u8]) -> Option<Self> {
        match s {
            b"bin" => Some(Self::Bin),
            b"elf" => Some(Self::Elf),
            _ => None,
        }
    }
}

/// An error which might occur when looking for the entry point of the init program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPointError {
    /// The header of the provided ELF file is invalid.
    InvalidElfHeader,
    /// The format of the provided ELF file is not supported.
    ///
    /// This can happen if the ELF file is not 64-bit, or if the target endianness is not the same
    /// as the host endianness. It's also possible that the ELF file is not executable.
    UnsupportElfFormat,
}

/// Attempts to guess the type of the provided file.
pub fn guess_type(file: &[u8]) -> Option<FileType> {
    match file {
        [0x7F, b'E', b'L', b'F', ..] => Some(FileType::Elf),
        _ => None,
    }
}

/// Parses the provided file to find its entry point.
///
/// The returned offset is relative to the provided file.
pub fn parse_entry_point(ty: FileType, file: &[u8]) -> Result<usize, EntryPointError> {
    match ty {
        FileType::Elf => parse_elf(file),
        FileType::Bin => Ok(0),
    }
}

/// Parses the provided ELF file to find its entry point.
fn parse_elf(file: &[u8]) -> Result<usize, EntryPointError> {
    const ELF_HEADER_SIZE: usize = 64;
    const ELFCLASS64: u8 = 2;
    const ET_EXEC: u16 = 2;

    let len = file.len();

    if len < ELF_HEADER_SIZE {
        return Err(EntryPointError::InvalidElfHeader);
    }

    if !matches!(file, [0x7F, b'E', b'L', b'F', ..]) {
        return Err(EntryPointError::InvalidElfHeader);
    }

    unsafe {
        let file = file.as_ptr();

        let elf_class = *file.add(4);
        if elf_class != ELFCLASS64 {
            return Err(EntryPointError::UnsupportElfFormat);
        }

        let elf_data = *file.add(5);

        #[cfg(target_endian = "little")]
        const ELFDATA: u8 = 1;
        #[cfg(target_endian = "big")]
        const ELFDATA: u8 = 2;

        if elf_data != ELFDATA {
            return Err(EntryPointError::UnsupportElfFormat);
        }

        let elf_type = *(file.add(16) as *const u16);
        if elf_type != ET_EXEC {
            return Err(EntryPointError::UnsupportElfFormat);
        }

        const ENTRY_POINT_OFFSET: usize = 16 + 2 + 2 + 4; // e_ident + e_type + e_machine + e_version

        // SAFETY:
        //  We made sure that the file is large enough to contain the entry point.
        let entry_point_bytes = &*(file.add(ENTRY_POINT_OFFSET) as *const [u8; 8]);
        let entry_point = u64::from_ne_bytes(*entry_point_bytes);

        if entry_point == 0 {
            return Err(EntryPointError::InvalidElfHeader);
        }

        if entry_point >= len as u64 {
            return Err(EntryPointError::InvalidElfHeader);
        }

        Ok(entry_point as usize)
    }
}
