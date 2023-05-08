use core::arch::asm;

use neodym_sys_common::{SysResult, SystemCall};

/// This function is called when the `syscall` instruction is executed in userland.
///
/// # Arguments
///
/// * `rax` contains the system call number
///
/// * `rdi` contains the first argument
///
/// * `rsi` contains the second argument
///
/// * `rdx` contains the third argument
///
/// # Return Value
///
/// The return value of the system call is stored in `rax`.
///
/// # Safety
///
/// This function is unsafe. The return address must be stored in `rcx` before calling the
/// function. This is normally done by the `syscall` instruction.
#[naked]
pub unsafe extern "C" fn handle_syscall() -> usize {
    unsafe {
        asm!(
            r#"
            push rcx
            mov rcx, rax
            call nd_handle_syscall_inner
            pop rcx
            sysretq
            "#,
            options(noreturn)
        );
    }
}

/// The Rust function responsible for handling system calls. This function is called by the
/// assembly function [`handle_syscall`].
///
/// The system call number is taken as the third parameter (ecx) because that's the first register
/// that is clobbered by the `syscall` instruction. Other registers are already properly setup by
/// the caller (in userspace).
#[inline(always)] // no sure whether the compiler can inline this. it would be nice.
#[no_mangle]
extern "C" fn nd_handle_syscall_inner(
    _arg0: usize,
    _arg1: usize,
    _arg2: usize,
    no: usize,
) -> SysResult {
    let Some(sysno) = SystemCall::from_usize(no) else {
        todo!("support invalid system calls");
    };

    match sysno {
        SystemCall::Terminate => {
            todo!("implement the Terminate system call");
        }
        SystemCall::MapMemory => {
            todo!("implement the MapMemory system call");
        }
    }
}
