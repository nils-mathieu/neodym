//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

/// The type of file to be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Let the kernel guess the type of the file.
    ///
    /// Note that the kernel will never guess the 'bin' file type.
    Guess = 1,
    /// The process will start by jumping at the begining of the file.
    Bin,
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
            b"guess" => Some(Self::Guess),
            b"bin" => Some(Self::Bin),
            b"elf" => Some(Self::Elf),
            _ => None,
        }
    }
}

/// An error which might occur when looking for the entry point of the init program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPointError {
    /// The provided [`FileType`] was [`FileType::Guess`], but the format of the file could not be
    /// guessed.
    CantGuess,
    /// The header of the provided ELF file
    InvalidElfHeader,
}

/// Parses the provided file to find its entry point.
///
/// The returned offset is relative to the provided file.
pub fn parse_entry_point(_ty: FileType, _file: &[u8]) -> Result<usize, EntryPointError> {
    Ok(0)
}
