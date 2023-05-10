//! This module re-export the implementation of every ststem call.

use core::arch::asm;
use core::mem::size_of;

use neodym_sys_common::{SysError, SysResult, SystemCall};

mod ring0;
mod terminate;

type SyscallFn = extern "C" fn(usize, usize, usize) -> SysResult;

/// This table is used by the `handle_syscall` function to dispatch the system call to the correct
/// function.
static ND_SYSTEM_CALL_TABLE: [SyscallFn; SystemCall::COUNT] = [ring0::ring0, terminate::terminate];

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
/// The return value of the system call is stored in `rax` and is of type [`SysResult`].
///
/// # Safety
///
/// This function is unsafe. The return address must be stored in `rcx` before calling the
/// function. This is normally done by the `syscall` instruction.
#[naked]
pub unsafe extern "C" fn handle_syscall() {
    unsafe {
        // NOTE:
        //  We use `rax` to communicate the return address of the system call. Because the C
        //  ABI stores the return value in `rax`, we've got nothing to do more than calling the
        //  function.
        //
        //  Similarly, the registers `rdi`, `rsi` and `rdx` are used to pass the arguments to the
        //  system calls, and are *coincedentally* the same as the C ABI. This means that we
        //  won't need to move any of those registers.
        //
        //  The `rcx` register contains the return address of the system call, so we need to save
        //  this one, as it can be clobbered by functions using the C ABI.
        asm!(
            r#"
            cmp       rax,   {}
            jae       1f
            push      rcx
            lea       rcx,   [{} + rax * {}]
            call      [rcx]
            pop       rcx
            sysretq
        1:
            mov rax, {}
            sysretq
            "#,
            const SystemCall::COUNT,
            sym ND_SYSTEM_CALL_TABLE,
            const size_of::<SyscallFn>(),
            const SysError::INVALID_ARGUMENT.0,
            options(noreturn)
        );
    }
}
