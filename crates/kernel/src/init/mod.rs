//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

mod elf;

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
    UnsupportedElfFormat,
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
pub fn find_entry_point(ty: FileType, file: &[u8]) -> Result<usize, EntryPointError> {
    match ty {
        FileType::Elf => self::elf::find_elf_entry_point(file),
        FileType::Bin => Ok(0),
    }
}
