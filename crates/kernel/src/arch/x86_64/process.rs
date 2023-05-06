use nd_x86_64::VirtAddr;

use super::MemoryMapper;

/// The part of a process's metadata that's specific to the **x86_64** architecture.
pub struct Process {
    /// The memory mapper used to allocate memory pages to the process.
    pub memory_mapper: MemoryMapper,
    /// The instruction pointer of the process, within its own address space.
    pub instruction_pointer: VirtAddr,
}
