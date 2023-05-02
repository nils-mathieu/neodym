//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

/// Loads the provided file as the first userspace program.
///
/// The file is assumed to be a flat binary, and the control is transferred to it at its very
/// first byte. This is fundamentally unsafe, as the kernel has no way to know whether the file
/// is actually a valid program. We'll have to trust the user on that.
///
/// The function never returns. If the program returns control to the kernel, an error will be
/// logged and the machine will be halted.
pub fn load(file: &[u8]) -> ! {
    nd_log::info!("Transfering control to the `nd_init` program...");

    // The `nd_init` program is not supposed to return control to the kernel.
    // We could encode that in the type system by making the signature of this function return
    // `!`, but that would make the function pretty useable when written in C, and getting the
    // control flow back would instantly trigger undefined behavior.
    let entry_point_ptr = file.as_ptr();
    let entry_point: extern "C" fn() = unsafe { core::mem::transmute(entry_point_ptr) };

    entry_point();

    nd_log::error!("`nd_init` has returned control to the kernel.");
    crate::arch::die();
}
