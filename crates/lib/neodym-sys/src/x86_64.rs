//! Raw system calls on the x86_64 architecture.

use core::arch::asm;

use neodym_sys_common::x86_64::SystemCall;
use neodym_sys_common::SysResult;

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

/// Terminates the current process.
///
/// This corresponds to the [`SystemCall::TerminateSelf`] system call.
pub fn terminate_self() -> ! {
    unsafe {
        // This system call is infallible won't even return.
        let _ = syscall0(SystemCall::TerminateSelf);
        core::hint::unreachable_unchecked();
    }
}
