use bitflags::bitflags;
use core::fmt;
use core::ops::{Index, IndexMut};

use crate::{IstIndex, PrivilegeLevel, SegmentSelector};

/// The address of an
/// [Interrupt Service Routine](https://wiki.osdev.org/Interrupt_Service_Routines).
pub type HandlerAddr = u64;

/// The kind of a gate descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GateType {
    /// The gate is an "interrupt" gate.
    ///
    /// During the execution of the *Interrupt Service Routine*, interrupts will be automatically
    /// disabled.
    Interrupt = 0b1110,
    /// The gate is a "trap" gate.
    ///
    /// Unlike *interrupt* gates, interrupts are not automatically disabled during the execution of
    /// the *Interrupt Service Routine*.
    Trap = 0b1111,
}

/// Stores a [64-bit gate descriptor](https://wiki.osdev.org/IDT#Gate_Descriptor_2).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct GateDescriptor {
    low: u64,
    high: u64,
}

impl GateDescriptor {
    /// The "null" gate descriptor.
    pub const NULL: Self = Self::from_raw(0, 0);

    /// Creates a new [`GateDescriptor`] from the provided 64-bit "high" and "low" 64-bit words.
    #[inline(always)]
    pub const fn from_raw(low: u64, high: u64) -> Self {
        Self { low, high }
    }

    /// Returns the inner 64-bit value of this gate descriptor.
    #[inline(always)]
    pub const fn to_raw(self) -> [u64; 2] {
        [self.low, self.high]
    }

    /// Creates a new [`GateDescriptor`].
    ///
    /// # Arguments
    ///
    /// - `offset`: The address of the
    /// [Interrupt Service Routine](https://wiki.osdev.org/Interrupt_Service_Routines) which will
    /// be called when the interrupt arrives.
    /// - `selector`: The code selector in which the routine is defined.
    /// - `ist`: If set, specifies the stack that should be used when executing the routine.
    /// - `ty`: The type of the gate. This is mainly used to specify whether interrupts should be
    /// disabled during the execution of the routine.
    /// - `dpl`: The privilege levels at which it is possible to execute the routine. Values lower
    /// than this value cannot execute the routine. Note that hardware interrupts ignore this
    /// mechanism.
    /// - `present`: Whether the gate descriptor is present at all.
    pub const fn new(
        offset: HandlerAddr,
        selector: SegmentSelector,
        ist: Option<IstIndex>,
        ty: GateType,
        dpl: PrivilegeLevel,
        present: bool,
    ) -> Self {
        let ist = if let Some(idx) = ist { idx as u64 } else { 0 };

        let mut ret = Self::NULL;

        ret.high |= offset >> 32;
        ret.low |= (offset & 0xFFFF0000) << 32;
        ret.low |= offset & 0xFFFF;
        ret.low |= (selector.to_raw() as u64) << 16;
        ret.low |= ist << 32;
        ret.low |= (ty as u64) << 40;
        ret.low |= (dpl as u64) << 45;
        ret.low |= (present as u64) << 47;

        ret
    }

    /// Returns the address of the
    /// [Interrupt Service Routine](https://wiki.osdev.org/Interrupt_Service_Routines) specified by
    /// this gate descriptor.
    #[inline(always)]
    pub const fn offset(self) -> HandlerAddr {
        let mut ret = 0;

        ret |= (self.high & 0xFFFFFFFF) << 32;
        ret |= (self.low & 0xFFFF0000_00000000) >> 32;
        ret |= self.low & 0xFFFF;

        ret
    }

    /// Returns the [`SegmentSelector`] that contains the *Interrupt Service Routine* of this
    /// gate descriptor.
    #[inline(always)]
    pub const fn selector(self) -> SegmentSelector {
        let raw = (self.low & 0xFFFF0000) >> 16;
        SegmentSelector::from_raw(raw as u16)
    }

    /// Returns the index within the *Interrupt Stack Stable* describing whether a specific stack
    /// should be used when calling the *Interrupt Service Routine* of this gate descriptor.
    #[inline(always)]
    pub const fn ist_index(self) -> Option<IstIndex> {
        let raw = (self.low >> 32) & 0b111;
        if raw == 0 {
            None
        } else {
            Some(unsafe { IstIndex::from_raw_unchecked(raw as u8) })
        }
    }

    /// Returns the [`GateType`] of this *gate descriptor*.
    ///
    /// # Errors
    ///
    /// The bit-field that is supposed to store this value may contain an invalid bit pattern. In
    /// that case, this function returns [`None`].
    #[inline(always)]
    pub const fn ty(self) -> Option<GateType> {
        let raw = (self.low >> 40) & 0b1111;
        match raw {
            0b1110 => Some(GateType::Interrupt),
            0b1111 => Some(GateType::Trap),
            _ => None,
        }
    }

    /// The privilege levels that are allowed to execute the *Interrupt Service Routine* specified
    /// by this gate descriptor.
    #[inline(always)]
    pub const fn dpl(self) -> PrivilegeLevel {
        let raw = (self.low >> 45) & 0b11;
        unsafe { PrivilegeLevel::from_raw_unchecked(raw as u8) }
    }

    /// Returns whether the present bit is set for this gate descritor.
    #[inline(always)]
    pub const fn present(self) -> bool {
        (self.low >> 47) & 1 != 0
    }
}

impl fmt::Debug for GateDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.low == 0 && self.high == 0 {
            return write!(f, "NULL");
        }

        let ty = match self.ty() {
            Some(GateType::Interrupt) => "Interrupt",
            Some(GateType::Trap) => "Trap",
            None => "InvalidType",
        };

        f.debug_struct("GateDescriptor")
            .field("offset", &format_args!("{:#x}", self.offset()))
            .field("selector", &self.selector())
            .field("ist_index", &self.ist_index())
            .field("ty", &format_args!("{ty}"))
            .field("dpl", &self.dpl())
            .field("present", &self.present())
            .finish()
    }
}

/// An [Interrupt Descriptor Table](https://wiki.osdev.org/IDT).
///
/// # Representation
///
/// The *Interrupt Descriptor Table* is an array of 256 [`GateDescriptor`]s. Each entry specifies
/// how to handle a specific interrupts.
///
/// The first 32 entries are used to handle CPU exceptions (i.e. page fault, divide by zero, etc).
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Ist([GateDescriptor; 256]);

/// A specific [CPU Exception](https://wiki.osdev.org/Exceptions).
///
/// # Types Of Exceptions
///
/// - **Faults** can be corrected and the program may continue as if nothing happened.
/// - **Traps** are reported immediately after the execution of the trapping instruction.
/// - **Aborts** indicates a severe, unrecoverable error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CpuException {
    /// Occurs when dividing any number by 0, or when the result of the division is too large to be
    /// represented in any destination.
    ///
    /// This exception is a **fault**.
    DivisionError = 0x00,
    /// This exception is a **trap**.
    Debug = 0x01,
    /// An hardware-driven interrupt that goes directly into the CPU (in which case it cannot be
    /// masked), or comes from another controller (in which case it *can* be masked).
    ///
    /// <https://wiki.osdev.org/Non_Maskable_Interrupt>
    NonMaskableInterrupt = 0x02,
    /// Occurs when the **INT3** instruction is executed.
    ///
    /// This exception is a **trap**.
    Breakpoint = 0x03,
    /// Occurs when the INTO instruction is executed while the overflow bit in **RFLAGS** is set.
    ///
    /// This exception is a **trap**.
    Overflow = 0x04,
    /// Occurs when a **BOUND** instruction is executed and the index is out of bounds.
    ///
    /// This exception is a **fault**.
    BoundRangeExceeded = 0x05,
    /// Occurs when the processor tries to execute an invalid or undefined op-code.
    ///
    /// This exception is a **fault**.
    InvalidOpCode = 0x06,
    /// Occurs when an *Floating Point Unit* (FPU) instruction is attempted but there is no FPU, or
    /// the FPU have been disabled.
    ///
    /// This exception is a **fault**.
    DeviceNotAvailable = 0x07,
    /// Occurs when an exception is unhandled or when an exception occurs while the CPU is trying to
    /// call the exception handler.
    ///
    /// This exception is an **abort**.
    ///
    /// This exception pushes an error code: it is always set to zero.
    DoubleFault = 0x08,
    /// Occurs when an invalid segment selector is referenced as part of a task switch, or as a
    /// result of a control transfer through a gate descriptor, which results in an invalid
    /// stack-segment reference in the TSS.
    ///
    /// This exception is a **fault**.
    ///
    /// This exception provides an error code: the [`TableEntryError`] which caused the
    /// exception.
    InvalidTSS = 0x0A,
    /// Occurs when trying to load a segment or gate with a *present*-bit set to `0`.
    ///
    /// Note that in the case of an absent stack selector, a
    /// [`StackSegmentFault`](CpuException::StackSegmentFault) occurs.
    ///
    /// This exception is a **fault**.
    ///
    /// This exception provides an error code: the [`TableEntryError`] that caused the
    /// exception.
    SegmentNotPresent = 0x0B,
    /// Occurs when:
    ///
    /// 1. Loading a stack segment referencing a segment descriptor which is not present.
    /// 2. Any **PUSH** or **POP** instruction, or any instruction using **ESP** or **EBP** as a
    /// base register, while the stack address is not in canonical form.
    /// 3. When the stack-limit check fails.
    ///
    /// This exception is a **fault**.
    ///
    /// This exception provides an error code: if non-zero, it stores the [`TableEntryError`]
    /// that wasn't present.
    StackSegmentFault = 0x0C,
    /// Occurs for various reasons (segmentation errors, executing privilege instructions outside
    /// of ring 0, etc).
    ///
    /// This exception is a **fault**.
    ///
    /// This exception provides an error code: if the exception was segment-related, it is set to
    /// the [`TableEntryError`] of that segment. Otherwise, 0.
    GeneralProtectionFault = 0x0D,
    /// Occurs when:
    ///
    /// 1. A [page directory or table](https://wiki.osdev.org/Paging) entry is not present in
    /// physical memory.
    /// 2. A protection check fails.
    ///
    /// This exception is a **fault**.
    ///
    /// This exception provides an error code: a set of flags describing why the page fault occured.
    /// It is represented by the [`PageFaultError`] type.
    PageFault = 0x0E,
    /// Occurs when a waiting floating-point instruction is executed while:
    ///
    /// 1. The [CR0.NE](https://wiki.osdev.org/CR0#CR0) flag is set.
    /// 2. An unmasked x87 floating point exception is pending.
    ///
    /// This exception is a **fault**.
    X87FloatingPointException = 0x10,
    /// Occurs when alignment checking is enabled and unaligned memory data reference is performed.
    ///
    /// This exception is a **fault**.
    ///
    /// This exception pushes an error code.
    AlignmentCheck = 0x11,
    /// This exception is model-specific and processor implementations are not required to support
    /// it. This exception usually occurs when the processor detects an internal error, such as
    /// bad memory, bus errors, cache errors, etc.
    ///
    /// This exception is an **abort**.
    MachineCheck = 0x12,
    /// Occurs when an unmasked 128-bit media floating point exception occurs and the
    /// [CR4.OSXMMEXCPT](https://wiki.osdev.org/CR4) is set. If that flag is clear, a
    /// [`CpuException::InvalidOpCode`] will be raised instead of this.
    ///
    /// This exception is a **fault**.
    SimdFloatingPointException = 0x13,
    /// This exception is a **fault**.
    VirtualizationException = 0x14,
    /// This exception is a **fault**.
    ///
    /// This exception pushes an error code.
    ControlProtectionException = 0x15,
    /// This exception is a **fault**.
    HypervisorInjectionException = 0x1C,
    /// This exception is a **fault**.
    ///
    /// This exception pushes an error code.
    VmmCommunicationException = 0x1D,
    /// This exception is a **fault**.
    ///
    /// This exception pushes an error code.
    SecurityException = 0x1E,
}

impl Index<CpuException> for Ist {
    type Output = GateDescriptor;

    #[inline(always)]
    fn index(&self, index: CpuException) -> &Self::Output {
        &self[index as u8]
    }
}

impl IndexMut<CpuException> for Ist {
    #[inline(always)]
    fn index_mut(&mut self, index: CpuException) -> &mut Self::Output {
        &mut self[index as u8]
    }
}

impl Index<u8> for Ist {
    type Output = GateDescriptor;

    #[inline(always)]
    fn index(&self, index: u8) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<u8> for Ist {
    #[inline(always)]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

/// A table in which a [`TableEntryError`] can occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableEntryKind {
    /// The entry index references a descriptor in the *Global Descriptor Table*.
    Gdt,
    /// The entry index references a descriptor in the *Interrupt Descriptor Table*.
    Idt,
    /// The entry index references a descriptor in the *Lobal Descriptor Table*.
    Ldt,
}

/// Describes which table entry produced an exception.
///
/// <https://wiki.osdev.org/Exceptions#Selector_Error_Code>
#[derive(Clone, Copy)]
pub struct TableEntryError(u32);

impl TableEntryError {
    /// Returns the index of the segment that produced the error.
    #[inline(always)]
    pub const fn index(self) -> u16 {
        (self.0 as u16) >> 3
    }

    /// Returns the table associated with the segment.
    #[inline]
    pub const fn table(self) -> TableEntryKind {
        match (self.0 >> 1) & 0b11 {
            0b00 => TableEntryKind::Gdt,
            0b01 => TableEntryKind::Idt,
            0b10 => TableEntryKind::Ldt,
            0b11 => TableEntryKind::Idt,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Returns whether the exception originated externally to the processor.
    #[inline(always)]
    pub const fn external(self) -> bool {
        self.0 & 1 != 0
    }
}

impl fmt::Debug for TableEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentSelectorError")
            .field("index", &self.index())
            .field("table", &self.table())
            .field("external", &self.external())
            .finish()
    }
}

bitflags! {
    /// An error specified by the CPU on [page fault](CpuException::PageFault).
    #[derive(Debug, Clone, Copy)]
    pub struct PageFaultError: u32 {
        /// The page fault was caused by a page-protection violation. When clear, it was caused by
        /// a non-present page.
        const PRESENT = 1 << 0;
        /// The page fault was caused by a write access. If clear, it was caused by a read access.
        const WRITE = 1 << 1;
        /// The page fault occured in ring 3. This does not necessarily mean that the page fault
        /// was a privilege violation.
        const USER = 1 << 2;
        /// One or more page directory entries contain reserved bits which are set to 1.
        const RESERVED_WRITE = 1 << 3;
        /// The page fault was caused by an instruction fetch. This only applies when the no-execute
        /// bit is supported and enabled.
        const INSTRUCTION_FETCH = 1 << 4;
        /// The page fault was caused by a protection-key violation. The **PKRU** register (for
        /// user-mode accesses) or **PKRS MSR** (for supervisor-mode accesses) specifies the
        /// protection-key rights.
        const PROTECTION_KEY = 1 << 5;
        /// The page fault was caused by a shadow stack access.
        const SHADOW_STACK = 1 << 6;
        /// The page fault was caused by a
        /// [*Software Guard Extension*](https://en.wikipedia.org/wiki/Software_Guard_Extensions).
        /// The fault is unrelated to ordinary paging.
        const SOFTWARE_GUARD_EXT = 1 << 7;
    }
}
