use core::arch::asm;

/// Halts the CPU until a new interrupt occurs.
#[inline(always)]
pub unsafe fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, preserves_flags, nostack));
    }
}

/// Enables interrupts.
#[inline(always)]
pub unsafe fn sti() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

/// Disables interrupts.
#[inline(always)]
pub unsafe fn cli() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}
