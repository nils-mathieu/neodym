//! Raw system calls on the x86_64 architecture.

use core::arch::asm;

use neodym_sys_common::x86_64::SystemCall;
use neodym_sys_common::{ProcessHandle, SysResult};

/// Performs a system call with no arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall0(n: SystemCall) -> SysResult {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n.to_usize(),
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    SysResult(ret)
}

/// Performs a system call with one arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall1(n: SystemCall, arg0: usize) -> SysResult {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n.to_usize(),
            in("rdi") arg0,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    SysResult(ret)
}

/// Performs a system call with two arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall2(n: SystemCall, arg0: usize, arg1: usize) -> SysResult {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n.to_usize(),
            in("rdi") arg0,
            in("rsi") arg1,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    SysResult(ret)
}

/// Performs a system call with three arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall3(n: SystemCall, arg0: usize, arg1: usize, arg2: usize) -> SysResult {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n.to_usize(),
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    SysResult(ret)
}

/// Terminates a specific process.
///
/// This function removes the process from the scheduler's queue, and frees all the resources
/// associated with it.
///
/// This corresponds to the [`SystemCall::Terminate`] system call.
#[inline(always)]
pub fn terminate(process: ProcessHandle) {
    unsafe { syscall1(SystemCall::Terminate, process.get()) };
}

/// Terminates the current process.
///
/// This function removes the current process from the scheduler's queue, and frees all the
/// resources associated with it.
///
/// This corresponds to the [`SystemCall::Terminate`] system call.
pub fn terminate_self() -> ! {
    unsafe {
        // We're not using the `syscall1` function here because we want to use the `noreturn`
        // option.

        core::arch::asm!(
            "syscall",
            in("rax") SystemCall::Terminate.to_usize(),
            in("rdi") 0,
            options(noreturn, nostack, preserves_flags)
        );
    }
}

/// Yields the control of the CPU to another, specific, process.
///
/// The amount of CPU time yielded to the process is the remainder of the current time slice of
/// the current process.
///
/// This corresponds to the [`SystemCall::Yield`] system call.
#[inline(always)]
pub fn yield_to(process: ProcessHandle) -> SysResult {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        syscall1(SystemCall::Yield, process.get())
    }
}

/// Yields the control of the CPU to any other process.
///
/// The scheduler will chose the process to yield CPU time to.
///
/// The amount of CPU time yielded to the process is the remainder of the current time slice of
/// the current process.
///
/// This corresponds to the [`SystemCall::Yield`] system call.
#[inline(always)]
pub fn yield_to_any() -> SysResult {
    unsafe { syscall1(SystemCall::Yield, 0) }
}
