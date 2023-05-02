//! Raw system calls.

use core::arch::asm;

/// The system call number for the `nd::sched::yield` system call.
pub const YIELD: usize = 0;
/// The system call number for the `nd::sched::terminate` system call.
pub const TERMINATE: usize = 1;

/// Performs a system call with no arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Performs a system call with one arguments.
///
/// # Safety
///
/// System calls are fundamentally unsafe. The specific safety requirement of this function depend
/// on the system call being performed.
#[inline(always)]
pub unsafe fn syscall1(n: usize, arg0: usize) -> usize {
    let ret: usize;

    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg0,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    ret
}
