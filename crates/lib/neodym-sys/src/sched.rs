//! System calls to interact with the scheduler.
//!
//! # The Neodym Scheduler
//!
//! Neodym uses a simple "round-robin" preemptive scheduler. This means that the kernel will
//! periodically interrupt the currently running process or task and give control to another one.
//!
//! One defining feature, however, is that processes are responsible for allocating CPU time
//! themselves. They can trade CPU time for responsiveness, or responsiveness for longer time
//! slices.

use crate::raw;
use crate::ProcessHandle;

use core::arch::asm;

/// Yields the control of the CPU to another, specific, process.
///
/// This corresponds to the `nd::sched::yield` system call.
#[inline(always)]
pub fn yield_to(process: ProcessHandle) {
    unsafe { raw::syscall1(raw::YIELD, process.get()) };
}

/// Yields the control of the CPU to any other process.
///
/// The scheduler will chose the process to yield CPU time to.
///
/// This corresponds to the `nd::sched::yield` system call.
#[inline(always)]
pub fn yield_to_any() {
    unsafe { raw::syscall1(raw::YIELD, 0) };
}

/// Terminates a specific process.
///
/// This function removes the process from the scheduler's queue, and frees all the resources
/// associated with it.
///
/// This corresponds to the `nd::sched::terminate` system call.
#[inline(always)]
pub fn terminate(process: ProcessHandle) {
    unsafe { raw::syscall1(raw::TERMINATE, process.get()) };
}

/// Terminates the current process.
///
/// This function removes the process from the scheduler's queue, and frees all the resources
/// associated with it.
///
/// This corresponds to the `nd::sched::terminate` system call.
pub fn terminate_self() -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") raw::TERMINATE,
            in("rdi") 0,
            options(noreturn, nostack, preserves_flags)
        );
    }
}
