//! This module contains the logic to initialize the first userspace program.
//!
//! This program is usually loaded as a kernel module by the bootloader.

use crate::arch::x86_64::OutOfPhysicalMemory;
use crate::process::Process;

/// The virtual address at which the first userspace program is loaded.
pub const LOAD_ADDRESS: usize = 0x10_0000;

/// Loads the provided file as the first userspace program.
///
/// The file is assumed to be a flat binary, and the control is transferred to it at its very
/// first byte. This is fundamentally unsafe, as the kernel has no way to know whether the file
/// is actually a valid program. We'll have to trust the user on that.
///
/// # Safety
///
/// The page allocator must be initialized.
///
/// The scheduler must be initialized.
pub unsafe fn load_init_program(file: &[u8]) -> Result<(), OutOfPhysicalMemory> {
    nd_log::info!("Starting the `nd_init` program...");

    // The program must be loaded at the address `0x10_0000` (1 Mb), and its entry point is exactly
    // at this address.
    let mut init = Process {
        #[cfg(target_arch = "x86_64")]
        x86_64: crate::arch::x86_64::Process {
            instruction_pointer: LOAD_ADDRESS as nd_x86_64::VirtAddr,
            memory_mapper: unsafe { crate::arch::x86_64::MemoryMapper::new().unwrap() },
        },
    };

    #[cfg(target_arch = "x86_64")]
    {
        use nd_x86_64::{PageTableFlags, VirtAddr};

        use crate::arch::x86_64::MappingError;

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;

        let mut remainder = file;
        let mut page_addr = LOAD_ADDRESS as VirtAddr;
        while !remainder.is_empty() {
            let entry = match init.x86_64.memory_mapper.create_mapping(page_addr, flags) {
                Ok(entry) => entry,
                Err(MappingError::AlreadyMapped(_)) => {
                    debug_assert!(false, "unreachable: this page should not be mapped");
                    unsafe { core::hint::unreachable_unchecked() };
                }
                Err(MappingError::OutOfPhysicalMemory) => return Err(OutOfPhysicalMemory),
            };

            let page = entry.kernel_virtual_address() as *mut u8;

            let to_copy = core::cmp::min(0x1000, remainder.len());
            unsafe { core::ptr::copy_nonoverlapping(remainder.as_ptr(), page, to_copy) };

            remainder = unsafe { remainder.get_unchecked(to_copy..) };
            page_addr += 0x1000;
        }
    }

    unsafe { super::spawn(init) };

    Ok(())
}
