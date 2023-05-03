use nd_array::Slab;
use nd_spin::Mutex;

pub mod scheduler;

/// Stores information about a process.
pub struct Process {}

/// The list of all processes currently running on the system.
static PROCESSES: Mutex<Slab<Process, 1024>> = Mutex::new(Slab::new());
