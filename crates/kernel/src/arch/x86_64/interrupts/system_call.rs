use core::arch::asm;

/// This function is called when the `syscall` instruction is executed in userland. In that case,
#[naked]
pub extern "C" fn handle_syscall(arg0: u64, arg1: u64, arg2: u64) {
    unsafe {
        asm!(
            r#"
            mov rcx, rax
            call handle_syscall_inner
            sysret
            "#,
            options(noreturn)
        );
    }
}

#[inline(always)] // no sure whether the compiler can inline this. it would be nice.
#[no_mangle]
extern "C" fn handle_syscall_inner(_arg0: u64, _arg1: u64, _arg2: u64, no: usize) -> usize {
    todo!("Implement system calls (no = {})", no);
}
