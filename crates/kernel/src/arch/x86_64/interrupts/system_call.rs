use core::arch::asm;

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

#[inline(always)] // no sure whether the compiler can inline this. it would be nice.
#[no_mangle]
extern "C" fn nd_handle_syscall_inner(
    _arg0: usize,
    _arg1: usize,
    _arg2: usize,
    no: usize,
) -> usize {
    todo!("Implement system calls (no = {})", no);
}
