#![allow(clippy::missing_safety_doc)]

use bitflags::bitflags;

use crate::{PhysAddr, PrivilegeLevel, SegmentSelector, VirtAddr};

use core::arch::asm;
use core::fmt;

/// Returns the current value of the stack pointer.
#[inline(always)]
pub fn rsp() -> VirtAddr {
    let rsp: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) rsp, options(nostack, nomem, preserves_flags));
    }
    rsp
}

/// Sets the value of the **RSP** register.
#[inline(always)]
pub unsafe fn set_rsp(rsp: VirtAddr) {
    unsafe {
        asm!("mov rsp, {}", in(reg) rsp, options(nomem, preserves_flags));
    }
}

/// Returns the value of the stack base pointer.
#[inline(always)]
pub fn rbp() -> VirtAddr {
    let rbp: u64;
    unsafe {
        asm!("mov {}, rbp", out(reg) rbp, options(nostack, nomem, preserves_flags));
    }
    rbp
}

/// Sets the value of the **RBP** register.
#[inline(always)]
pub unsafe fn set_rbp(rbp: VirtAddr) {
    unsafe {
        asm!("mov rbp, {}", in(reg) rbp, options(nomem, preserves_flags));
    }
}

/// Returns the current value of the instruction pointer.
#[inline(always)]
pub fn rip() -> u64 {
    let rip: u64;
    unsafe {
        asm!("lea {}, [rip]", out(reg) rip, options(nostack, nomem, preserves_flags));
    }
    rip
}

bitflags! {
    /// The flags that the CPU keeps track of.
    #[derive(Debug, Clone, Copy)]
    pub struct RFlags: u64 {
        /// Set by the CPU if the last arithmetic operation resulted in a carry out of the
        /// most-significant bit of the result.
        const CARRY = 1 << 0;
        /// Set by the CPU if the last result has an even number of 1 bits (this flag is not set
        /// for all operations).
        const PARITY = 1 << 2;
        /// Set by the CPU if the last arithmetic operation resulted in a carry out of bit 3 of the
        /// result.
        const AUXILIARY_CARRY = 1 << 4;
        /// Set by the CPU if the last arithmetic resulted in a zero.
        const ZERO = 1 << 6;
        /// Set by the CPU if the last arithmetic operation resulted in a negative number.
        const SIGN = 1 << 7;
        /// Enables signle-step mode for debugging.
        const TRAP = 1 << 8;
        /// Enables interrupts.
        const INTERRUPT = 1 << 9;
        /// Determines the direction of string instructions.
        const DIRECTION = 1 << 10;
        /// Set by the CPU when the sign bit of the reuslt of the last signed integer operation
        /// differs from the source operand.
        const OVERFLOW = 1 << 11;
        /// Used by `iret` in hardware task switch mode to determine if the current task is nested.
        const NESTED_TASK = 1 << 14;
        /// Allows to restart an instruction following an instruction breakpoint.
        const RESUME = 1 << 16;
        /// Enables virtual-8086 mode.
        const VIRTUAL_8086 = 1 << 17;
        /// Enables automatic alignment checking. Only works in ring 3.
        const ALIGNMENT_CHECK = 1 << 18;
        ///
        const VIRTUAL_INTERRUPT = 1 << 19;
        /// Indicates that an external maskable interrupt is pending.
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        /// If this flag is modifiable, then the CPUID instruction is supported.
        const ID = 1 << 21;
    }
}

/// Returns the value of the **RFLAGS** register,
#[inline(always)]
pub unsafe fn rflags() -> RFlags {
    unsafe {
        let ret: u64;
        asm!("pushfq; pop {}", out(reg) ret, options(nomem, nostack, preserves_flags));
        RFlags::from_bits_retain(ret)
    }
}

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
    /// Note that this is only applicable if the [`Cr4::PCID`] flag is clear.
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
pub fn cr3() -> Cr3 {
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

/// The value of **AMD**'s **STAR** register.
#[derive(Clone, Copy)]
pub struct Star(u64);

impl Star {
    /// The value of the **STAR** register.
    pub const MSR: u32 = 0xC000_0081;

    /// Creates a new `Star` value.
    #[inline(always)]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw value of the **STAR** register.
    #[inline(always)]
    pub const fn to_raw(self) -> u64 {
        self.0
    }

    /// Creates a new [`Star`] instance from the given segment selectors indexes.
    ///
    /// # Arguments
    ///
    /// - `sysret_base`: Specifies both the **CS** and **SS** segment selectors to be loaded when
    ///   the **SYSRET** instruction is executed. Specifically, **CS** will be set to this
    ///   segment index plus 2, and **SS** will be set to this segment index plus 1.
    ///
    /// - `syscall_base`: Specifies both the **CS** and **SS** segment selectors to be loaded when
    ///   the **SYSCALL** instruction is executed. Specifically, **CS** will be set to this
    ///   segment, and **SS** will be set to this segment index plus 1.
    ///
    /// # Panics
    ///
    /// In debug builds, this function panics if the segment selectors do not have a **RPL** of
    /// 0 and 3 respectively.
    #[inline(always)]
    pub const fn new(sysret_base: SegmentSelector, syscall_base: SegmentSelector) -> Self {
        debug_assert!(
            matches!(
                syscall_base.requested_privilege_level(),
                PrivilegeLevel::Ring0
            ),
            "syscall_base should have an RPL of 0"
        );
        debug_assert!(
            matches!(
                sysret_base.requested_privilege_level(),
                PrivilegeLevel::Ring3
            ),
            "sysret_base should have an RPL of 3"
        );

        let sysret = sysret_base.to_raw() as u64;
        let syscall = syscall_base.to_raw() as u64;

        Self(syscall << 32 | sysret << 48)
    }

    /// Returns the segment selector for the **CS** segment.
    #[inline(always)]
    pub const fn cs_syscall(&self) -> SegmentSelector {
        let t = (self.0 >> 32) as u16;
        SegmentSelector::from_raw(t)
    }

    /// Returns the segment selector for the **SS** segment.
    #[inline(always)]
    pub const fn ss_syscall(&self) -> SegmentSelector {
        let t = (self.0 >> 32) as u16;
        SegmentSelector::from_raw(t + 8)
    }

    /// Returns the segment selector for the **CS** segment.
    #[inline(always)]
    pub const fn cs_sysret(&self) -> SegmentSelector {
        let t = (self.0 >> 48) as u16;
        SegmentSelector::from_raw(t + 16)
    }

    /// Returns the segment selector for the **CS** segment.
    #[inline(always)]
    pub const fn ss_sysret(&self) -> SegmentSelector {
        let t = (self.0 >> 48) as u16;
        SegmentSelector::from_raw(t + 8)
    }
}

impl fmt::Debug for Star {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Star")
            .field("cs_syscall", &self.cs_syscall())
            .field("ss_syscall", &self.ss_syscall())
            .field("cs_sysret", &self.cs_sysret())
            .field("ss_sysret", &self.ss_sysret())
            .finish()
    }
}

/// The value of **AMD**'s **STAR** register.
#[inline(always)]
pub fn star() -> Star {
    unsafe { Star::from_raw(crate::rdmsr(Star::MSR)) }
}

/// Sets the value of the **STAR** register.
#[inline(always)]
pub unsafe fn set_star(star: Star) {
    unsafe {
        crate::wrmsr(Star::MSR, star.to_raw());
    }
}

const LSTAR: u32 = 0xC000_0082;

/// The value of **AMD**'s **LSTAR** register.
///
/// This is the instruction pointer that will be loaded when the **SYSCALL** instruction is
/// executed.
#[inline(always)]
pub fn lstar() -> VirtAddr {
    unsafe { crate::rdmsr(LSTAR) }
}

/// Sets the value of the **LSTAR** register.
#[inline(always)]
pub unsafe fn set_lstar(lstar: VirtAddr) {
    unsafe {
        crate::wrmsr(LSTAR, lstar);
    }
}

bitflags! {
    /// A possible value of **INTEL**'s **IA32_EFER** register (Extended Feature Enable Register).
    #[derive(Debug, Clone, Copy)]
    pub struct Ia32Efer: u64 {
        /// Enables the `syscall` and `sysret` instructions, for compatibility with AMD
        /// processors.
        const SYSTEM_CALL_ENABLE = 1 << 0;

        /// Enables IA-32e mode operation.
        const IA32_MODE_ENABLE = 1 << 8;

        /// Set when the IA32e mode is active.
        const IA32_MODE_ENABLE_ACTIVE = 1 << 10;

        ///
        const EXECUTE_DISABLE = 1 << 11;
    }
}

const IA32_EFER: u32 = 0xC000_0080;

/// Returns the value of the **IA32_EFER** register.
#[inline(always)]
pub fn ia32_efer() -> Ia32Efer {
    unsafe { Ia32Efer::from_bits_retain(crate::rdmsr(IA32_EFER)) }
}

/// Sets the value of the **IA32_EFER** register.
#[inline(always)]
pub unsafe fn set_ia32_efer(efer: Ia32Efer) {
    unsafe {
        crate::wrmsr(IA32_EFER, efer.bits());
    }
}
