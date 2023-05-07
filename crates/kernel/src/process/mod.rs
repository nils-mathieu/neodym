mod init;

pub use self::init::*;

/// Represents a process running on the system.
pub struct Process {
    /// The architecture-specific part of the process.
    #[cfg(target_arch = "x86_64")]
    x86_64: crate::arch::x86_64::Process,
}

/// Spawns a new process.
///
/// # Safety
///
/// The global process scheduler must have been initialized previously.
pub unsafe fn spawn(mut state: Process) {
    unsafe { crate::arch::x86_64::setup_process(&mut state.x86_64) };
    todo!();
}
