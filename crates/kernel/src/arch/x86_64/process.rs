use core::arch::asm;

use nd_x86_64::VirtAddr;

use super::{MemoryMapper, OutOfPhysicalMemory, PageAllocatorTok};

/// The part of a process's metadata that's specific to the **x86_64** architecture.
pub struct Process {
    /// The memory mapper used to allocate memory pages to the process.
    pub memory_mapper: MemoryMapper,
    /// The saved instruction pointer of the process, within its own address space. Note that this
    /// value isn't updated in real-time.
    pub instruction_pointer: VirtAddr,
}

/// Initializes the provided userspace process to be spawned. This is the initialization function
/// for the **x86_64** architecture.
///
/// # Steps
///
/// 1. Switch the address space to the process's address space.
pub fn spawn(state: Process) -> Result<(), OutOfPhysicalMemory> {
    // Map the kernel address space into the program.

    unsafe {
        asm!(
            r#"
            sysret
            "#,
            in ("rcx") state.instruction_pointer,
            options(noreturn)
        );
    }
}
