//! This module defines the initialization logic of the **GDT** and **IDT**. System calls are
//! initialized here as well.

use core::mem::size_of_val;

use nd_x86_64::{
    DescriptorTable, Efer, GateDescriptor, GateType, Idt, IstIndex, PrivilegeLevel,
    SegmentDescriptor, SegmentSelector, Star, TablePtr, Tss, VirtAddr,
};

/// The global descriptor table that we are going to load. We can't use a simple array because some
/// descriptors may take two slots.
#[repr(C)]
#[derive(Debug)]
struct Gdt {
    null: SegmentDescriptor<1>,
    kernel_code: SegmentDescriptor<1>,
    kernel_data: SegmentDescriptor<1>,
    user_code: SegmentDescriptor<1>,
    user_data: SegmentDescriptor<1>,
    tss: SegmentDescriptor<2>,
}

impl Gdt {
    /// A [`Gdt`] with all fields set to 0.
    pub const NULL: Self = Self {
        null: SegmentDescriptor::NULL,
        kernel_code: SegmentDescriptor::NULL,
        kernel_data: SegmentDescriptor::NULL,
        user_code: SegmentDescriptor::NULL,
        user_data: SegmentDescriptor::NULL,
        tss: SegmentDescriptor::NULL,
    };

    pub const KERNEL_CODE: SegmentSelector =
        SegmentSelector::new(1, DescriptorTable::Gdt, PrivilegeLevel::Ring0);
    pub const KERNEL_DATA: SegmentSelector =
        SegmentSelector::new(2, DescriptorTable::Gdt, PrivilegeLevel::Ring0);
    pub const TSS: SegmentSelector =
        SegmentSelector::new(5, DescriptorTable::Gdt, PrivilegeLevel::Ring0);

    /// Returns a [`TablePtr`] referencing this *Global Descriptor Table*.
    pub fn table_ptr(&self) -> TablePtr {
        let limit = core::mem::size_of::<Gdt>() as u16 - 1;
        let base = self as *const Self as VirtAddr;
        TablePtr { base, limit }
    }
}

/// The stack that will be used by the kernel.
static mut KERNEL_STACK: [u8; 4096 * 4] = [0u8; 4096 * 4];

/// The stack that will be used when a double fault occurs. This is required because a double
/// fault might occur because of a stack overflow, and in that case, the kernel stack would be
/// unusable.
static mut DOUBLE_FAULT_STACK: [u8; 4096 * 2] = [0u8; 4096 * 2];

/// The task state segment.
static mut TSS: Tss = Tss::new();

/// The *Global Descriptor Table* that we're going to use.
static mut GDT: Gdt = Gdt::NULL;

/// The *Interrupt Descriptor Table* that we're going to use.
static mut IDT: Idt = Idt::new();

/// Initializes the GDT.
///
/// # Safety
///
/// This function must only be called once.
pub unsafe fn setup_gdt() {
    unsafe {
        nd_log::trace!("Setting up the GDT...");
        TSS.set_interrupt_stack(
            IstIndex::One,
            DOUBLE_FAULT_STACK
                .as_ptr()
                .add(size_of_val(&DOUBLE_FAULT_STACK)) as usize as VirtAddr,
        );
        TSS.set_stack_pointer(
            PrivilegeLevel::Ring0,
            KERNEL_STACK.as_ptr().add(size_of_val(&KERNEL_STACK)) as usize as VirtAddr,
        );

        GDT.kernel_code = SegmentDescriptor::code(true, PrivilegeLevel::Ring0, false, true);
        GDT.kernel_data = SegmentDescriptor::data(true, PrivilegeLevel::Ring0, false, true);
        GDT.user_data = SegmentDescriptor::data(true, PrivilegeLevel::Ring3, false, true);
        GDT.user_code = SegmentDescriptor::code(true, PrivilegeLevel::Ring3, false, true);
        GDT.tss = SegmentDescriptor::tss(
            true,
            PrivilegeLevel::Ring0,
            &TSS as *const Tss as usize as u64,
        );

        nd_x86_64::lgdt(&GDT.table_ptr());
        nd_x86_64::set_cs(Gdt::KERNEL_CODE);
        nd_x86_64::set_ss(Gdt::KERNEL_DATA);
        nd_x86_64::ltr(Gdt::TSS);
    }
}

/// Sets the IDT up.
///
/// # Safety
///
/// This function should only be called once.
pub unsafe fn setup_idt() {
    nd_log::trace!("Setting up the IDT...");

    unsafe {
        macro_rules! set_exception_handler {
            ($f:ident, $handler:expr) => {
                IDT.$f(
                    $handler,
                    Gdt::KERNEL_CODE,
                    None,
                    GateType::Trap,
                    PrivilegeLevel::Ring0,
                );
            };
        }
        macro_rules! set_interrupt_handler {
            ($index:expr, $handler:expr) => {
                IDT[$index] = GateDescriptor::new(
                    $handler as usize as u64,
                    Gdt::KERNEL_CODE,
                    None,
                    GateType::Interrupt,
                    PrivilegeLevel::Ring0,
                    true,
                );
            };
        }

        set_exception_handler!(set_division_error, super::interrupts::division_error);
        set_exception_handler!(set_breakpoint, super::interrupts::breakpoint);
        set_exception_handler!(
            set_bound_range_exceeded,
            super::interrupts::bound_range_exceeded
        );
        set_exception_handler!(set_invalid_op_code, super::interrupts::invalid_op_code);
        set_exception_handler!(
            set_device_not_available,
            super::interrupts::device_not_available
        );
        IDT.set_double_fault(
            super::interrupts::double_fault,
            Gdt::KERNEL_CODE,
            Some(IstIndex::One),
            GateType::Trap,
            PrivilegeLevel::Ring0,
        );
        set_exception_handler!(set_invalid_tss, super::interrupts::invalid_tss);
        set_exception_handler!(
            set_segment_not_present,
            super::interrupts::segment_not_present
        );
        set_exception_handler!(
            set_stack_segment_fault,
            super::interrupts::stack_segment_fault
        );
        set_exception_handler!(
            set_general_protection_fault,
            super::interrupts::general_protection_fault
        );
        set_exception_handler!(set_page_fault, super::interrupts::page_fault);
        set_exception_handler!(
            set_x87_floating_point_exception,
            super::interrupts::x87_floating_point_exception
        );
        set_exception_handler!(set_alignment_check, super::interrupts::alignment_check);
        set_exception_handler!(set_machine_check, super::interrupts::machine_check);
        set_exception_handler!(
            set_simd_floating_point_exception,
            super::interrupts::simd_floating_point_exception
        );
        set_exception_handler!(
            set_virtualization_exception,
            super::interrupts::virtualization_exception
        );
        set_exception_handler!(
            set_control_protection_exception,
            super::interrupts::control_protection_exception
        );
        set_exception_handler!(
            set_hypervisor_injection_exception,
            super::interrupts::hypervisor_injection_exception
        );
        set_exception_handler!(
            set_vmm_communication_exception,
            super::interrupts::vmm_communication_exception
        );
        set_exception_handler!(
            set_security_exception,
            super::interrupts::security_exception
        );

        set_interrupt_handler!(32, super::interrupts::apic_timer);
        set_interrupt_handler!(39, super::interrupts::apic_spurious);

        nd_x86_64::lidt(&IDT.table_ptr());
    }
}

/// Initializes the necessary registers to make system calls work.
///
/// This includes enabling the extended feature enable register for compatibility between Intel
/// and AMD processors, setting the STAR and LSTAR registers.
///
/// # Safety
///
/// This function should only be called once.
pub unsafe fn setup_system_calls() {
    nd_log::trace!("Setting up system calls...");

    unsafe {
        nd_x86_64::set_efer(nd_x86_64::efer() | Efer::SYSTEM_CALL_ENABLE);
        nd_x86_64::set_star(Star::new(
            SegmentSelector::new(2, DescriptorTable::Gdt, PrivilegeLevel::Ring3),
            SegmentSelector::new(1, DescriptorTable::Gdt, PrivilegeLevel::Ring0),
        ));
        nd_x86_64::set_lstar(super::interrupts::handle_syscall as usize as VirtAddr);
    }
}
