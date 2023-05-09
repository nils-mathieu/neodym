//! Raw system calls on the x86_64 architecture.

use core::arch::asm;
use core::mem::MaybeUninit;

use neodym_sys_common::{MemorySegment, PageTableEntry, SysResult, SystemCall};

use crate::ProcessHandle;

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
/// This corresponds to the [`SystemCall::Terminate`] system call.
#[inline(always)]
pub fn terminate_self() -> ! {
    unsafe {
        // This system call is infallible won't even return as we're passing a null process handle.
        let _ = syscall1(SystemCall::Terminate, 0);
        core::hint::unreachable_unchecked();
    }
}

/// Terminates the given process.
///
/// If the povided process handle is `None`, or a handle to the current process, then the current
/// process is terminated and the function never returns.
///
/// This corresponds to the [`SystemCall::Terminate`] system call.
#[inline(always)]
pub fn terminate(process: Option<ProcessHandle>) -> SysResult {
    unsafe { syscall1(SystemCall::Terminate, process.map_or(0, ProcessHandle::get)) }
}

/// Initializes `buf` with a list of physical memory segments available.
///
/// The total number of segments available on the system is returned, regardless of the length of
/// `buf`.
#[inline(always)]
pub fn get_memory(buf: &mut [MaybeUninit<MemorySegment>]) -> SysResult {
    unsafe { syscall2(SystemCall::GetMemory, buf.as_mut_ptr() as usize, buf.len()) }
}

/// Maps memory into the address space of the given process.
///
/// This corresponds to the [`SystemCall::MapMemory`] system call.
#[inline(always)]
pub fn map_memory(entries: &[PageTableEntry]) -> SysResult {
    unsafe {
        syscall2(
            SystemCall::MapMemory,
            entries.as_ptr() as usize,
            entries.len(),
        )
    }
}
