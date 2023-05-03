//! Provides ways to interact with the Local APIC of the current CPU.

use nd_x86_64::{PhysAddr, VirtAddr};

/// The address of the `IA32_APIC_BASE` MSR.
pub const IA32_APIC_BASE: u32 = 0x1B;

/// Returns the base address of the local XAPIC.
///
/// The function accesses the `IA32_APIC_BASE` MSR to get the base address of the local XAPIC.
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_xapic_base() -> PhysAddr {
    unsafe { nd_x86_64::rdmsr(IA32_APIC_BASE) & 0xFFFFF000 }
}

/// Sets the base address of the local XAPIC.
///
/// This function sets the base address of the local XAPIC by writing to the `IA32_APIC_BASE` MSR.
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn set_xapic_base(base: PhysAddr) {
    unsafe { nd_x86_64::wrmsr(IA32_APIC_BASE, base) };
}

/// Hardware-enables the local APIC by reloading the `IA32_APIC_BASE` MSR.
#[allow(clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn hardware_enable_xapic() {
    unsafe { set_xapic_base(get_xapic_base()) };
}

/// A XAPIC register.
#[repr(align(16))]
struct Register(u32);

impl Register {
    /// Reads the value of the register.
    #[inline(always)]
    pub fn read(&self) -> u32 {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    /// Writes a value to the register.
    #[inline(always)]
    pub fn write(&mut self, value: u32) {
        unsafe { core::ptr::write_volatile(&mut self.0, value) }
    }
}

/// A possible Local APIC divide configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimerDivisor {
    /// Divide by 2.
    Div2 = 0,
    /// Divide by 4.
    Div4 = 1,
    /// Divide by 8.
    Div8 = 2,
    /// Divide by 16.
    Div16 = 3,
    /// Divide by 32.
    Div32 = 8,
    /// Divide by 64.
    Div64 = 9,
    /// Divide by 128.
    Div128 = 10,
    /// Divide by 256.
    Div256 = 11,
}

impl TimerDivisor {
    /// Attempts to convert a raw integer into a [`TimerDivisor`].
    pub const fn from_register(val: u32) -> Option<Self> {
        match val {
            0 => Some(Self::Div2),
            1 => Some(Self::Div4),
            2 => Some(Self::Div8),
            3 => Some(Self::Div16),
            8 => Some(Self::Div32),
            9 => Some(Self::Div64),
            10 => Some(Self::Div128),
            11 => Some(Self::Div256),
            _ => None,
        }
    }
}

/// An possible operation mode for the LAPIC timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimerMode {
    /// One-shot mode.
    OneShot = 0,
    /// Periodic mode.
    Periodic = 1,
    /// TSC-deadline mode.
    Deadline = 2,
}

/// Represents the registers of a Local APIC.
#[repr(C)]
struct Registers {
    _reserved0: [Register; 2],
    id: Register,
    version: Register,
    _reserved1: [Register; 4],
    task_priority: Register,
    arbitrary_priority: Register,
    processor_priority: Register,
    end_of_interrupt: Register,
    remote_read: Register,
    logical_destination: Register,
    destination_format: Register,
    spurious_interrupt_vector: Register,
    in_service: [Register; 8],
    trigger_mode: [Register; 8],
    interrupt_request: [Register; 8],
    error_status: Register,
    _reserved2: [Register; 6],
    lvt_corrected_machine_check_interrupt: Register,
    interrupt_command: [Register; 2],
    lvt_timer: Register,
    lvt_thermal_sensor: Register,
    lvt_performance_monitoring_counters: Register,
    lvt_lint0: Register,
    lvt_lint1: Register,
    lvt_error: Register,
    initial_count: Register,
    current_count: Register,
    _reserved3: [Register; 4],
    divide_configuration: Register,
    _reserved4: Register,
}

/// Stores some information about the local XAPIC.
///
/// This structure is not meant to be kept around, it is only used to make sure some invariants
/// remain constant during configuration of the Local APIC. This avoids reading the MSR again
/// and again.
pub struct XApic<'a> {
    /// The base of the Local APIC.
    base: &'a mut Registers,
}

impl<'a> XApic<'a> {
    /// Returns a new `XApic` instance.
    ///
    /// # Safety
    ///
    /// The memory at the base address of the Local APIC must be mapped to an identity-mapped
    /// physical address.
    #[inline(always)]
    pub unsafe fn identity_mapped() -> Self {
        unsafe { Self::from_virtual_address(get_xapic_base()) }
    }

    /// Returns a new `XApic` instance from the provided virtual base address.
    ///
    /// # Safety
    ///
    /// The memory referenced by the provided virtual address must be valid for reads and writes
    /// and must remain logically borrowed by the `XApic` instance for the duration of its lifetime.
    #[inline(always)]
    pub unsafe fn from_virtual_address(addr: VirtAddr) -> Self {
        let base = &mut *(addr as *mut Registers);

        XApic { base }
    }

    /// Returns the LAPIC ID of the current CPU.
    #[inline(always)]
    pub fn id(&self) -> u8 {
        (self.base.id.read() >> 24) as u8
    }

    /// Signals the end of an interrupt to the local APIC.
    #[inline(always)]
    pub fn end_of_interrupt(&mut self) {
        self.base.end_of_interrupt.write(0);
    }

    /// Sets the divisor of the timer.
    #[inline(always)]
    pub fn set_timer_divisor(&mut self, divide: TimerDivisor) {
        self.base.divide_configuration.write(divide as u32);
    }

    /// Returns the divisor of the timer.
    #[inline(always)]
    pub fn timer_divisor(&self) -> u32 {
        self.base.divide_configuration.read()
    }

    /// Sets the initial count of the timer.
    #[inline(always)]
    pub fn set_timer_initial_count(&mut self, count: u32) {
        self.base.initial_count.write(count);
    }

    /// Returns the initial count of the timer.
    #[inline(always)]
    pub fn timer_initial_count(&self) -> u32 {
        self.base.initial_count.read()
    }

    /// Returns the current count of the timer.
    #[inline(always)]
    pub fn timer_current_count(&self) -> u32 {
        self.base.current_count.read()
    }

    /// Configures the timer of the local APIC.
    ///
    /// Interrupts will be fired on the specified index, using the specified mode.
    #[inline(always)]
    pub fn configure_timer(&mut self, index: u8, mode: TimerMode) {
        self.base
            .lvt_timer
            .write(index as u32 | (mode as u32) << 17);
    }

    /// Configures the spurious interrupt vector. This is also used to enable the local APIC.
    #[inline(always)]
    pub fn configure_spurious(&mut self, index: u8, apic_enable: bool) {
        self.base
            .spurious_interrupt_vector
            .write(index as u32 | (apic_enable as u32) << 8);
    }
}
