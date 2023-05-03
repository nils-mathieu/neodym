use core::cmp::Ordering;

use nd_array::BinaryHeap;
use neodym_sys_common::ProcessHandle;

/// A time slice that has been allocated for a specific process.
#[derive(Debug, Clone, Copy)]
pub struct Slice {
    /// The process that this slice has been allocated for.
    ///
    /// Note that it is possible for a process to yield the remainder of their slice to another
    /// process, in which case this field might not match the process that's actually running off
    /// this slice.
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
pub struct Scheduler {
    slices: BinaryHeap<Slice, { Self::MAX_SLICES }>,
}

impl Scheduler {
    /// The maximum number of slices which may be allocated at the same time.
    pub const MAX_SLICES: usize = 256;

    /// Allocates a new time slice for a process.
    ///
    /// # Errors
    ///
    /// This function may return `false` if too many slices have been allocated and the scheduler
    /// is out of memory.
    pub fn allocate(&mut self, slice: Slice) -> Result<(), ()> {
        match self.slices.push(slice) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }

    /// Notifies the scheduler that a time slice as expired and that another process should be
    /// scheduled.
    ///
    /// Note that it is possible for the same process to be scheduled twice in a row. The caller
    /// should make sure not to preempt the current process if it is still running.
    ///
    /// If the returned value is `None`, then there is no more processes to schedule.
    pub fn next(&mut self) -> Option<Slice> {
        self.slices.pop();
        self.slices.peek().copied()
    }
}
