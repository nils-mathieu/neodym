#![allow(clippy::missing_safety_doc)]

use crate::SegmentSelector;

use core::arch::asm;

/// Sets the value of the **CS** register.
#[inline]
pub unsafe fn set_cs(sel: SegmentSelector) {
    // adapted from
    //    https://docs.rs/x86_64/0.14.10/src/x86_64/instructions/segmentation.rs.html#69-82

    unsafe {
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(sel.to_raw()),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
    }
}

/// Returns the value of the **CS** register.
#[inline(always)]
pub fn cs() -> SegmentSelector {
    unsafe {
        let ret: u16;
        asm!("mov {:x}, cs", out(reg) ret, options(nomem, nostack, preserves_flags));
        SegmentSelector::from_raw(ret)
    }
}

/// Sets the value of the **SS** register.
#[inline(always)]
pub unsafe fn set_ss(sel: SegmentSelector) {
    unsafe {
        asm!("mov ss, {:x}", in(reg) sel.to_raw(), options(nostack, preserves_flags));
    }
}

/// Returns the value of thte **SS** register.
#[inline(always)]
pub fn ss() -> SegmentSelector {
    unsafe {
        let ret: u16;
        asm!("mov {:x}, ss", out(reg) ret, options(nomem, nostack, preserves_flags));
        SegmentSelector::from_raw(ret)
    }
}
