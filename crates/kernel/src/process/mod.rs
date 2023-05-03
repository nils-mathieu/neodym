use nd_array::Slab;

pub mod scheduler;

/// Stores information about a process.
pub struct Process {}

/// The list of all processes currently running on the system.
static PROCESSES: Slab<Process, 1024> = Slab::new();
