use super::EntryPointError;

/// The value of the first magic number of the `e_ident` field.
const EI_MAG0: u8 = 0x7F;
/// The value of the second magic number of the `e_ident` field.
const EI_MAG1: u8 = b'E';
/// The value of the third magic number of the `e_ident` field.
const EI_MAG2: u8 = b'L';
/// The value of the fourth magic number of the `e_ident` field.
const EI_MAG3: u8 = b'F';

/// The size of the `e_ident` field, in bytes.
const EI_NIDENT: usize = 16;
/// The magic number of the `e_ident` field indicating that the remainder of the file is a
/// 64-bit ELF file.
const ELFCLASS64: u8 = 0x02;
/// A value of the `e_type` field indicating that the file is an executable (opposed to a
/// shared object, for example).
const ET_EXEC: u16 = 2;
/// A value of the `e_machine` field indicating that the file is a 64-bit x86_64 ELF file.
const EM_X86_64: u16 = 0x3E;

/// The value that's expected in the `e_ident` field to indicate that the file is a little
/// endian ELF file.
#[cfg(target_endian = "little")]
const EI_DATA: u8 = 1; // ELFDATA2LSB
/// The value that's expected in the `e_ident` field to indicate that the file is a big
/// endian ELF file.
#[cfg(target_endian = "big")]
const EI_DATA: u8 = 2; // ELFDATA2MSB

/// The header of a 64-bit ELF file.
#[repr(C)]
struct ElfHdr64 {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

/// Parses the provided ELF file to find its entry point.
///
/// This functions checks the following properties of the input file:
///
/// 1. Whether the file is a valid ELF file.
/// 2. Whether the ELF format is supported.
/// 3. The position of its entry point.
pub fn find_elf_entry_point(file: &[u8]) -> Result<usize, EntryPointError> {
    let len = file.len();

    if len < core::mem::size_of::<ElfHdr64>() {
        return Err(EntryPointError::InvalidElfHeader);
    }

    // SAFETY:
    //  We just made sure that the file is at least as large as the size of the ELF header.
    let hdr = unsafe { &*(file.as_ptr() as *const ElfHdr64) };

    // Ensures that we're working with an ELF file.
    if !matches!(hdr.e_ident, [EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3, ..]) {
        return Err(EntryPointError::InvalidElfHeader);
    }

    // Check the remainder of the `e_ident` field, but return another error if case of failure.
    // Omitting the first four bytes (which have already been checked might help with
    // optimizations).
    if !matches!(hdr.e_ident, [_, _, _, _, ELFCLASS64, EI_DATA, ..],) {
        return Err(EntryPointError::UnsupportedElfFormat);
    }

    if hdr.e_type != ET_EXEC {
        return Err(EntryPointError::UnsupportedElfFormat);
    }

    if hdr.e_machine != EM_X86_64 {
        return Err(EntryPointError::UnsupportedElfFormat);
    }

    if hdr.e_entry == 0 {
        // We verified that the ELF file was executable, but no entry point was specified.
        // This probably means that the file is invalid. If the dynamic linker is expected to find
        // the entry point when fixing the relocations, then the ELF file is not supported.
        //
        // We don't have a dynamic linker hehe, and the init program is not supposed to have any
        // of that stuff.
        return Err(EntryPointError::UnsupportedElfFormat);
    }

    // Last sanity check that we can perform. Jumpting there is not any less safe than jumping at
    // any other address, but letting the user know that the provided ELF file is corrupted is
    // still a good idea.
    //
    // Or are they testing us?
    if hdr.e_entry >= len as u64 {
        return Err(EntryPointError::InvalidElfHeader);
    }

    Ok(hdr.e_entry as usize)
}
