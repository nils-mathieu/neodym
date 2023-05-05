/// A memory mapper is used to map virtual memory to physical memory.
///
/// Typically, each process, as well as the kernel has its own memory mapper. During each context
/// switch, the effective memory mapper is switched to the one of the process that is now being
/// executed.
pub struct MemoryMapper {}
