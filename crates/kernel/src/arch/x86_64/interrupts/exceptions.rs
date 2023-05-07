use nd_x86_64::{InterruptStackFrame, PageFaultError, TableEntryError};

pub extern "x86-interrupt" fn double_fault(_: InterruptStackFrame, _: u64) -> ! {
    panic!("Double Fault {:x}", nd_x86_64::rsp());
}

pub extern "x86-interrupt" fn invalid_op_code(frame: InterruptStackFrame) {
    panic!(
        "Invalid Op Code (addr = {:#x})",
        frame.instruction_pointer()
    );
}

pub extern "x86-interrupt" fn device_not_available(_: InterruptStackFrame) {
    panic!("Device Not Available");
}

pub extern "x86-interrupt" fn segment_not_present(_: InterruptStackFrame, err: TableEntryError) {
    panic!("Segment Not Present (err = {err:?})");
}

pub extern "x86-interrupt" fn stack_segment_fault(_: InterruptStackFrame, err: TableEntryError) {
    panic!("Stack Segment Fault (err = {err:?})");
}

pub extern "x86-interrupt" fn general_protection_fault(
    _: InterruptStackFrame,
    err: TableEntryError,
) {
    if err.to_raw() == 0 {
        panic!("General Protection Fault (err = None)");
    } else {
        panic!("General Protection Fault (err = {err:?})");
    }
}

pub extern "x86-interrupt" fn page_fault(_: InterruptStackFrame, err: PageFaultError) {
    let addr = nd_x86_64::cr2();
    panic!("Page Fault (err = {err:?}, addr = {addr:#x})");
}

pub extern "x86-interrupt" fn division_error(_: InterruptStackFrame) {
    panic!("Division Error");
}

pub extern "x86-interrupt" fn alignment_check(_: InterruptStackFrame, _: u64) {
    panic!("Alignment Check");
}

pub extern "x86-interrupt" fn machine_check(_: InterruptStackFrame) -> ! {
    panic!("Machine Check");
}

pub extern "x86-interrupt" fn invalid_tss(_: InterruptStackFrame, err: TableEntryError) {
    panic!("Invalid TSS (err = {err:?})");
}

pub extern "x86-interrupt" fn x87_floating_point_exception(_: InterruptStackFrame) {
    panic!("x87 Floating Point Exception");
}

pub extern "x86-interrupt" fn simd_floating_point_exception(_: InterruptStackFrame) {
    panic!("SIMD Floating Point Exception");
}

pub extern "x86-interrupt" fn virtualization_exception(_: InterruptStackFrame) {
    panic!("Virtualization");
}

pub extern "x86-interrupt" fn control_protection_exception(_: InterruptStackFrame, _: u64) {
    panic!("Control Protection Exception");
}

pub extern "x86-interrupt" fn hypervisor_injection_exception(_: InterruptStackFrame) {
    panic!("Hypervisor Injection Exception");
}

pub extern "x86-interrupt" fn vmm_communication_exception(_: InterruptStackFrame, _: u64) {
    panic!("VMM Communication Exception");
}

pub extern "x86-interrupt" fn security_exception(_: InterruptStackFrame, _: u64) {
    panic!("Security Exception");
}

pub extern "x86-interrupt" fn bound_range_exceeded(_: InterruptStackFrame) {
    panic!("Bound Range Exceeded");
}

pub extern "x86-interrupt" fn breakpoint(_: InterruptStackFrame) {
    nd_log::info!("BREAKPOINT");
}
