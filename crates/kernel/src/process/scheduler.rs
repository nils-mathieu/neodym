use core::cmp::Ordering;

use neodym_sys_common::ProcessHandle;

/// A time slice that has been allocated for a specific process.
pub struct Slice {
    /// The process that this slice has been allocated for.
    pub process: ProcessHandle,
    /// The number of ticks that were allocated for this slice.
    ///
    /// This number is decremented each time on each CPU tick, and when it reaches zero, the slice
    /// is considered to be expired and the process is preempted.
    pub ticks: u32,
    /// The position of this slice.
    ///
    /// This is the approximate number of ticks that the process is willing to wait before being
    /// scheduled.
    pub position: u32,
}

impl Slice {
    /// The expected end time of this slice.
    #[inline(always)]
    pub fn expected_end_time(&self) -> u32 {
        // Using a saturating add here avoids overflows and prevents processes from being
        // scheduled too early when using a large number of ticks to lie about their position.
        self.position.saturating_add(self.ticks)
    }
}

impl PartialEq for Slice {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.ticks == other.ticks && self.position == other.position
    }
}

impl PartialOrd for Slice {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.expected_end_time()
            .partial_cmp(&other.expected_end_time())
    }
}

/// The scheduler is responsible for scheduling processes on the CPU.
///
/// The scheduler is a simple round-robin scheduler, which allocates a fixed number of ticks to
/// each process, and then switches to the next process when the time slice expires.
///
/// Each scheduler is responsible for scheduling processes on a single CPU.
///
/// # Implementation
///
/// The scheduler is implemented using a priority queue, itself implemented as a binary heap.
pub struct Scheduler {}

impl Scheduler {
    /// The maximum number of slices which may be allocated at the same time.
    pub const MAX_SLICES: usize = 256;

    /// Allocates a new time slice for this process.
    ///
    /// # Arguments
    ///
    /// - `process` is the handle to the process that has allocated this time slice.
    ///
    /// - `position` is the maximum number of ticks that this process is willing to wait before
    /// the slice is executed. This is a *hint*, meaning that the scheduler may decide to execute
    /// it earlier or later if its more convenient.
    ///
    /// - `ticks` the number of ticks that the process wants to be allocated.
    ///
    /// # Errors
    ///
    /// This function may return `false` if too many slices have been allocated and the scheduler
    /// is out of memory.
    #[must_use = "This function may fail if the scheduler is out of memory."]
    pub fn allocate(&mut self, slice: Slice) -> bool {
        todo!();
    }

    /// Notifies the scheduler that a tick has passed.
    ///
    /// If this function returns [`Some(_)`], then a new process should be scheduled.
    ///
    /// Note that it is possible for the same process to be scheduled twice in a row. The caller
    /// should make sure not to preempt the current process if it is still running.
    pub fn tick(&mut self) -> Option<ProcessHandle> {
        todo!();
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        todo!();
    }
}
