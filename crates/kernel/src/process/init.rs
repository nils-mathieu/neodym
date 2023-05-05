//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

/// Loads the provided file as the first userspace program.
///
/// The file is assumed to be a flat binary, and the control is transferred to it at its very
/// first byte. This is fundamentally unsafe, as the kernel has no way to know whether the file
/// is actually a valid program. We'll have to trust the user on that.
pub fn load_init_program(file: &[u8]) {
    nd_log::info!("Starting the `nd_init` program...");

    // The program must be loaded at the address `0x10_0000` (1 Mb), and its entry point is exactly
    // at this address.
}
