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

/// Performs a write to the provided I/O port.
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

/// Performs a read on the provided I/O port.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let ret: u8;
        asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack, preserves_flags));
        ret
    }
}
