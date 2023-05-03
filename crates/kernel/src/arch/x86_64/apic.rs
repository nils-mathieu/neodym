use nd_apic::{TimerDivisor, TimerMode, XApic};

/// Initializes the local APIC of the current CPU.
///
/// # Safety
///
/// This function should only be called once per CPU.
///
/// The local APIC must be identiy mapped.
pub unsafe fn initialize_lapic() {
    unsafe {
        nd_apic::hardware_enable_xapic();
    }

    let mut lapic = unsafe { XApic::identity_mapped() };

    lapic.configure_spurious(39, true);

    lapic.configure_timer(32, TimerMode::Periodic);
    lapic.set_timer_divisor(TimerDivisor::Div2);
    lapic.set_timer_initial_count(u32::MAX / 8);
}
