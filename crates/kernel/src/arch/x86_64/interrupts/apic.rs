use nd_apic::XApic;
use nd_x86_64::InterruptStackFrame;

pub extern "x86-interrupt" fn apic_timer(_: InterruptStackFrame) {
    // SAFETY:
    //  The APIC is identity mapped. Because local APICs are CPU-local, we can safely access the
    //  APIC from any CPU as long as service handlers are not recursively called (because that
    //  would break aliasing).
    let mut lapic = unsafe { XApic::identity_mapped() };

    lapic.end_of_interrupt();
}

pub extern "x86-interrupt" fn apic_spurious(_: InterruptStackFrame) {
    // We don't need to send an EOI here.
}
