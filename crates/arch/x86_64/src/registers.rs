#![allow(clippy::missing_safety_doc)]

use bitflags::bitflags;

use crate::{PhysAddr, SegmentSelector, VirtAddr};

use core::arch::asm;
use core::fmt;

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

bitflags! {
    /// The flags that the **CR0** register might hold.
    pub struct Cr0: u64 {
        /// Whether the CPU is running in protected mode.
        const PROTECTED_MODE = 1 << 0;
        /// Enables monitoring the coprocessor.
        ///
        /// When this flag is set, some instructions related to the coprocessor will trigger an
        /// exception.
        const MONITOR_COPROCESSOR = 1 << 1;
        /// Forces all x87 floating-point instructions to produce an exception.
        const EMULATE_COPROCESSOR = 1 << 2;
        /// Automatically set to 1 by the processor on *hardware* context switches.
        const TASK_SWITCHED = 1 << 3;
        /// Indicates that some math instructions are available in the CPU.
        ///
        /// This is normally set for all modern CPUs.
        const EXTENSION_TYPE = 1 << 4;
        /// Enables the native error reporting mechanism for floating-point errors.
        const NUMERIC_ERROR = 1 << 5;
        /// Controls whether supervisor-level writes to read-only pages are inhibited.
        ///
        /// When set, it is not possible to write to read-only pages from ring 0.
        const WRITE_PROTECT = 1 << 16;
        /// Enables automatic usermode alignment checking if [`RFlags::ALIGNMENT_CHECK`] is also
        /// set.
        const ALIGNMENT_MASK = 1 << 18;
        /// Ignored, should always remain unset.
        ///
        /// Older CPUs used this to control write-back/write-through cache strategy.
        const NOT_WRITE_THROUGH = 1 << 29;
        /// Disables some processor caches.
        ///
        /// The behavior of this flag is model dependent.
        const CACHE_DISABLE = 1 << 30;
        /// Enables paging.
        ///
        /// If this bit is set, [`PROTECTED_MODE_ENABLE`](Cr0Flags::PROTECTED_MODE_ENABLE) must be
        /// set as well.
        const PAGING = 1 << 31;
    }
}

/// Sets the value of the **CR0** register.
#[inline(always)]
pub unsafe fn set_cr0(cr0: Cr0) {
    unsafe {
        asm!("mov cr0, {}", in(reg) cr0.bits(), options(nostack, preserves_flags));
    }
}

/// Returns the value of the **CR0** register.
#[inline(always)]
pub fn cr0() -> Cr0 {
    unsafe {
        let ret: u64;
        asm!("mov {}, cr0", out(reg) ret, options(nostack, preserves_flags));
        Cr0::from_bits_retain(ret)
    }
}

/// Returns the value of the **CR2** register.
#[inline(always)]
pub fn cr2() -> VirtAddr {
    let ret: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) ret, options(nostack, preserves_flags));
    }
    ret
}

bitflags! {
    /// The flags that the **CR3** register might hold.
    ///
    /// Those flags are only applicable if the [`Cr4::PCID`] flag is set.
    #[derive(Debug, Clone, Copy)]
    pub struct Cr3Flags: u64 {
        /// Use a writethrough cache policy for the P4 table. When left clear, a writeback policy
        /// is used instead.
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        /// Disables caching for the P4 table.
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

/// The content of the **CR3** register.
#[derive(Clone, Copy)]
pub struct Cr3(u64);

impl Cr3 {
    /// Creates a new instance of the [`Cr3`] structure from the raw value of the register.
    #[inline(always)]
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw value of the register.
    #[inline(always)]
    pub fn to_raw(self) -> u64 {
        self.0
    }

    /// Creates a new instance of the structure.
    #[inline(always)]
    pub fn new(addr: VirtAddr, flags: Cr3Flags) -> Self {
        Self(addr | flags.bits())
    }

    /// Returns the address of the P4 table.
    #[inline(always)]
    pub fn addr(self) -> PhysAddr {
        self.0 & 0x000f_ffff_ffff_f000
    }

    /// Returns the flags of the P4 table.
    ///
    /// Note that this is only applicable if the [`Cr4::PCIDE`] flag is clear.
    #[inline(always)]
    pub fn flags(self) -> Cr3Flags {
        Cr3Flags::from_bits_truncate(self.0)
    }

    /// Returns the value of the **PCID** field.
    ///
    /// Note that this is only applicable if the [`Cr4::PCID`] flag is set.
    #[inline(always)]
    pub fn pcid(self) -> u16 {
        (self.0 >> 12) as u16 & 0xfff
    }
}

impl fmt::Debug for Cr3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Cr3");
        s.field("addr", &self.addr());

        if cr4().contains(Cr4::PCID) {
            s.field("pcid", &self.pcid());
        } else {
            s.field("flags", &self.flags());
        }

        s.finish()
    }
}

/// Gets the value of the **CR3** register.
#[inline(always)]
pub fn get_cr3() -> Cr3 {
    let ret: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) ret, options(nostack, preserves_flags));
    }
    Cr3::from_raw(ret)
}

/// Sets the value of the **CR3** register.
#[inline(always)]
pub unsafe fn set_cr3(cr3: Cr3) {
    unsafe {
        asm!("mov cr3, {}", in(reg) cr3.to_raw(), options(nostack, preserves_flags));
    }
}

bitflags! {
    /// The flag that may be set in the **CR4** register.
    pub struct Cr4: u64 {
        /// Enables hardware-supported performance enhancements for software running in
        /// virtual-8086 mode.
        const VIRTUAL_8086_MODE_EXTENSIONS = 1 << 0;
        /// Enables support for protected-mode virtual interrupts.
        const PROTECTED_MODE_VIRTUAL_INTERRUPTS = 1 << 1;
        /// When set, only ring 0 can execute the `RDTSC` or `RDTSCP` instructions.
        const TIMESTAMP_DISABLE = 1 << 2;
        /// Enables I/O breakpoint capability and enforces treatment of `DR4` and `DR5` registers
        /// as reserved.
        const DEBUGGING_EXTENSIONS = 1 << 3;
        /// Enables the use of 4MB physical frames; ignored if
        /// [`PHYSICAL_ADDRESS_EXTENSION`](Cr4Flags::PHYSICAL_ADDRESS_EXTENSION)
        /// is set.
        ///
        /// This is always ignored in long mode.
        const PAGE_SIZE_EXTENSION = 1 << 4;
        /// Enables physical address extensions and 2MB physical frames. Required in long mode.
        const PHYSICAL_ADDRESS_EXTENSION = 1 << 5;
        /// Enables the machine-check exception mechanism.
        const MACHINE_CHECK_EXCEPTION = 1 << 6;
        /// Enables the global page feature, allowing some page translations to
        /// be marked as global (see [`PageTableFlags::GLOBAL`]).
        const PAGE_GLOBAL = 1 << 7;
        /// Allows software running at any privilege level to use the `RDPMC` instruction.
        const PERFORMANCE_MONITOR_COUNTER = 1 << 8;
        /// Enables the use of legacy SSE instructions; allows using `FXSAVE`/`FXRSTOR` for saving
        /// processor state of 128-bit media instructions.
        const OSFXSR = 1 << 9;
        /// Enables the SIMD floating-point exception (`#XF`) for handling unmasked 256-bit and
        /// 128-bit media floating-point errors.
        const OSXMMEXCPT_ENABLE = 1 << 10;
        /// Prevents the execution of the `SGDT`, `SIDT`, `SLDT`, `SMSW`, and `STR` instructions by
        /// user-mode software.
        const USER_MODE_INSTRUCTION_PREVENTION = 1 << 11;
        /// Enables 5-level paging on supported CPUs (Intel Only).
        const L5_PAGING = 1 << 12;
        /// Enables VMX instructions (Intel Only).
        const VIRTUAL_MACHINE_EXTENSIONS = 1 << 13;
        /// Enables SMX instructions (Intel Only).
        const SAFER_MODE_EXTENSIONS = 1 << 14;
        /// Enables software running in 64-bit mode at any privilege level to read and write
        /// the FS.base and GS.base hidden segment register state.
        const FSGSBASE = 1 << 16;
        /// Enables process-context identifiers (PCIDs).
        const PCID = 1 << 17;
        /// Enables extended processor state management instructions, including `XGETBV` and
        /// `XSAVE`.
        const OSXSAVE = 1 << 18;
        /// Enables the Key Locker feature (Intel Only).
        ///
        /// This enables creation and use of opaque AES key handles; see the
        /// [Intel Key Locker Specification](https://software.intel.com/content/www/us/en/develop/download/intel-key-locker-specification.html)
        /// for more information.
        const KEY_LOCKER = 1 << 19;
        /// Prevents the execution of instructions that reside in pages accessible by user-mode
        /// software when the processor is in supervisor-mode.
        const SUPERVISOR_MODE_EXECUTION_PROTECTION = 1 << 20;
        /// Enables restrictions for supervisor-mode software when reading data from user-mode
        /// pages.
        const SUPERVISOR_MODE_ACCESS_PREVENTION = 1 << 21;
        /// Enables protection keys for user-mode pages.
        ///
        /// Also enables access to the PKRU register (via the `RDPKRU`/`WRPKRU`
        /// instructions) to set user-mode protection key access controls.
        const PROTECTION_KEY_USER = 1 << 22;
        /// Enables Control-flow Enforcement Technology (CET)
        ///
        /// This enables the shadow stack feature, ensuring return addresses read
        /// via `RET` and `IRET` have not been corrupted.
        const CONTROL_FLOW_ENFORCEMENT = 1 << 23;
        /// Enables protection keys for supervisor-mode pages (Intel Only).
        ///
        /// Also enables the `IA32_PKRS` MSR to set supervisor-mode protection
        /// key access controls.
        const PROTECTION_KEY_SUPERVISOR = 1 << 24;
    }
}

/// Returns the value of the **CR4** register.
#[inline(always)]
pub fn cr4() -> Cr4 {
    unsafe {
        let ret: u64;
        asm!("mov {}, cr4", out(reg) ret, options(nostack, preserves_flags));
        Cr4::from_bits_retain(ret)
    }
}

/// Sets the value of the **CR4** register.
#[inline(always)]
pub unsafe fn set_cr4(cr4: Cr4) {
    unsafe {
        asm!("mov cr4, {}", in(reg) cr4.bits(), options(nostack, preserves_flags));
    }
}
