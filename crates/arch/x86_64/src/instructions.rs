use core::arch::asm;

use crate::VirtAddr;

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

/// References a table which may be loaded into the CPU with instructions such as [`lidt`] or
/// [`lgdt`].
#[repr(packed)]
pub struct TablePtr {
    /// The size of the table, usually minus one.
    pub limit: u16,
    /// The virtual address of the table.
    pub base: VirtAddr,
}

/// Loads a new *Interrupt Descriptor Table*.
#[inline(always)]
pub unsafe fn lidt(p: &TablePtr) {
    unsafe {
        asm!("lidt [{}]", in(reg) p, options(readonly, nostack, preserves_flags));
    }
}

/// Returns the currently loaded *Interrupt Descriptor Table*.
#[inline]
pub unsafe fn sidt() -> TablePtr {
    unsafe {
        let mut ret = TablePtr { limit: 0, base: 0 };
        asm!("sidt [{}]", in(reg) &mut ret, options(nostack, preserves_flags));
        ret
    }
}

/// Loads a new *Global Descriptor Table*.
#[inline(always)]
pub unsafe fn lgdt(p: &TablePtr) {
    unsafe {
        asm!("lgdt [{}]", in(reg) p, options(readonly, nostack, preserves_flags));
    }
}

/// Returns the currently loaded *Global Descriptor Table*.
#[inline]
pub unsafe fn sgdt() -> TablePtr {
    unsafe {
        let mut ret = TablePtr { limit: 0, base: 0 };
        asm!("sgdt [{}]", in(reg) &mut ret, options(nostack, preserves_flags));
        ret
    }
}
