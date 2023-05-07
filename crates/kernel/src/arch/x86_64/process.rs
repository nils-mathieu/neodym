use core::arch::asm;

use nd_x86_64::VirtAddr;

use super::{MappingError, MemoryMapper, OutOfPhysicalMemory};

/// The part of a process's metadata that's specific to the **x86_64** architecture.
pub struct Process {
    /// The memory mapper used to allocate memory pages to the process.
    pub memory_mapper: MemoryMapper,
    /// The saved instruction pointer of the process, within its own address space. Note that this
    /// value isn't updated in real-time.
    pub instruction_pointer: VirtAddr,
    /// The stack pointer of the process, within its own address space. Note that this value isn't
    /// updated in real-time.
    pub stack_pointer: VirtAddr,
}

/// Spawns the provided userspace process.
///
/// # Steps
///
/// 1. Map the kernel's address space into the process's address space.
///
/// 2. Switch the address space to the process's address space.
///
/// 3. Use SYSRET to jump to the process's entry point in userspace.
pub fn spawn(mut state: Process) -> Result<(), OutOfPhysicalMemory> {
    // Map the kernel into the process's address space.
    match state.memory_mapper.map_kernel() {
        Ok(()) => {}
        Err(MappingError::OutOfPhysicalMemory) => return Err(OutOfPhysicalMemory),
        Err(MappingError::AlreadyMapped(_)) => {
            // TODO:
            //  Figure out whether this should be unreachable or not.
            panic!("Kernel already mapped into the process's address space.")
        }
    }

    unsafe { state.memory_mapper.switch() };

    unsafe {
        asm!(
            r#"
            mov rsp, {}
            sysret
            "#,
            in(reg) state.stack_pointer,
            in("rcx") state.instruction_pointer,
            options(noreturn)
        );
    }
}
