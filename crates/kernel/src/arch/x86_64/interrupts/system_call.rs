use core::arch::asm;

/// This function is called when the `syscall` instruction is executed in userland.
#[naked]
pub extern "C" fn handle_syscall(arg0: usize, arg1: usize, arg2: usize) -> usize {
    unsafe {
        asm!(
            r#"
            mov rcx, rax
            call nd_handle_syscall_inner
            sysret
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
