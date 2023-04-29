use nd_x86_64::{
    CpuException, DescriptorTable, GateDescriptor, GateType, Idt, PrivilegeLevel,
    SegmentDescriptor, SegmentSelector, TablePtr, VirtAddr,
};

/// The global descriptor table that we are going to load. We can't use a simple array because some
/// descriptors may take two slots.
#[repr(C)]
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

    /// Returns a [`TablePtr`] referencing this *Global Descriptor Table*.
    pub fn table_ptr(&self) -> TablePtr {
        let limit = core::mem::size_of::<Gdt>() as u16 - 1;
        let base = self as *const Self as VirtAddr;
        TablePtr { base, limit }
    }
}

/// The *Global Descriptor Table* that we're going to use.
static mut GDT: Gdt = Gdt::NULL;

/// The *Interrupt Descriptor Table* that we're going to use.
static mut IDT: Idt = Idt::new();

/// Initializes the **GDT** and the **IDT**.
///
/// # Safety
///
/// This function must only be called once.
pub unsafe fn initialize() {
    unsafe {
        // Initialize the GDT.
        GDT.kernel_code = SegmentDescriptor::code(true, PrivilegeLevel::Ring0, false, true);
        GDT.kernel_data = SegmentDescriptor::data(true, PrivilegeLevel::Ring0, false, true);
        GDT.user_code = SegmentDescriptor::code(true, PrivilegeLevel::Ring3, false, true);
        GDT.user_data = SegmentDescriptor::data(true, PrivilegeLevel::Ring3, false, true);

        let cs = SegmentSelector::new(1, DescriptorTable::Gdt, PrivilegeLevel::Ring0);
        let ss = SegmentSelector::new(0, DescriptorTable::Gdt, PrivilegeLevel::Ring0);

        nd_x86_64::lgdt(&GDT.table_ptr());
        nd_x86_64::set_cs(cs);
        nd_x86_64::set_ss(ss);

        let trap = |addr: VirtAddr| {
            GateDescriptor::new(addr, cs, None, GateType::Trap, PrivilegeLevel::Ring0, true)
        };

        // Initialize the IDT.
        IDT[CpuException::DoubleFault] = trap(super::interrupts::double_fault as usize as u64);
        IDT[CpuException::InvalidOpCode] = trap(super::interrupts::invalid_op_code as usize as u64);
        IDT[CpuException::DeviceNotAvailable] =
            trap(super::interrupts::device_not_available as usize as u64);
        IDT[CpuException::SegmentNotPresent] =
            trap(super::interrupts::segment_not_present as usize as u64);
        IDT[CpuException::StackSegmentFault] =
            trap(super::interrupts::stack_segment_fault as usize as u64);
        IDT[CpuException::GeneralProtectionFault] =
            trap(super::interrupts::general_protection_fault as usize as u64);
        IDT[CpuException::PageFault] = trap(super::interrupts::page_fault as usize as u64);
        IDT[CpuException::DivisionError] = trap(super::interrupts::division_error as usize as u64);
        IDT[CpuException::InvalidTSS] = trap(super::interrupts::invalid_tss as usize as u64);
        IDT[CpuException::X87FloatingPointException] =
            trap(super::interrupts::x87_floating_point_exception as usize as u64);
        IDT[CpuException::AlignmentCheck] =
            trap(super::interrupts::alignment_check as usize as u64);
        IDT[CpuException::MachineCheck] = trap(super::interrupts::machine_check as usize as u64);
        IDT[CpuException::SimdFloatingPointException] =
            trap(super::interrupts::simd_floating_point_exception as usize as u64);
        IDT[CpuException::VirtualizationException] =
            trap(super::interrupts::virtualization_exception as usize as u64);
        IDT[CpuException::ControlProtectionException] =
            trap(super::interrupts::control_protection_exception as usize as u64);
        IDT[CpuException::HypervisorInjectionException] =
            trap(super::interrupts::hypervisor_injection_exception as usize as u64);
        IDT[CpuException::VmmCommunicationException] =
            trap(super::interrupts::vmm_communication_exception as usize as u64);
        IDT[CpuException::SecurityException] =
            trap(super::interrupts::security_exception as usize as u64);
        IDT[CpuException::Breakpoint] = trap(super::interrupts::breakpoint as usize as u64);

        nd_x86_64::lidt(&IDT.table_ptr());
    }
}
