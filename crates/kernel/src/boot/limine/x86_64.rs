//! Contains the Limine entry point on **x86_64**.
//!
//! See [`entry_point`].

use crate::arch::x86_64::{SysInfo, SysInfoTok};

use super::find_init_program;
use super::{BOOTLOADER_INFO, HHDM, KERNEL_ADDR, MEMORY_MAP};

const KERNEL_STACK_SIZE: usize = 4096 * 16;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

/// The entry point of the kernel when booted by the Limine bootloader on **x86_64**.
#[naked]
pub extern "C" fn entry_point() -> ! {
    unsafe {
        // We need to setup a stack within the kernel's address space, as the bootloader's memory
        // eventually gets reclaimed as usable memory.
        core::arch::asm!(
            r#"
            lea rsp, [{} + {}]
            mov rbp, rsp
            jmp {}
            "#,
            sym KERNEL_STACK,
            const KERNEL_STACK_SIZE,
            sym entry_point_inner,
            options(noreturn),
        );
    }
}

pub extern "C" fn entry_point_inner() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe { crate::arch::x86_64::initialize_logger() };

    //
    // Gather the responses from the Limine bootloader.
    // Some are necessary, others are just nice information to have.
    //

    if let Some(info) = BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})!", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader.");
    }

    let Some(kernel_addr) = KERNEL_ADDR.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::arch::die();
    };

    let Some(hhdm) = HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM offset.");
        crate::arch::die();
    };

    let Some(_memmap) = MEMORY_MAP.response() else {
        nd_log::error!("The Limine bootloader did not provide a map of the usable memory.");
        crate::arch::die();
    };

    let Some(_nd_init) = find_init_program() else {
        nd_log::error!("An `nd_init` module is expected along with the kernel.");
        nd_log::error!("Check your Limine config!");
        nd_log::error!("");
        nd_log::error!("Example `limine.cfg`:");
        nd_log::error!("");
        nd_log::error!("    PROTOCOL=limine");
        nd_log::error!("    KERNEL_PATH=boot:///neodym");
        nd_log::error!("    MODULE_PATH=boot:///nd_init");
        nd_log::error!("");
        crate::arch::die();
    };

    // Initialize the global kernel info object.
    //
    // This is used throughout the kernel to access information about the kernel and the system
    // that the kernel is running on.
    let _sys_info = unsafe {
        SysInfoTok::initialize(SysInfo {
            kernel_phys_addr: kernel_addr.physical_base(),
            kernel_virt_addr: kernel_addr.virtual_base(),
            kernel_size: crate::image_size(),
            hhdm_offset: hhdm.offset(),
        })
    };

    // Initialize the CPU in a well-known state.
    //
    // SAFETY:
    //  Thos function must only be called once. We're still in the entry point, which is only
    //  called once by the bootloader.
    unsafe {
        crate::arch::x86_64::setup_gdt();
        crate::arch::x86_64::setup_idt();
        crate::arch::x86_64::setup_system_calls();
        crate::arch::x86_64::initialize_lapic();

        // Enable interrupts. We're ready to be interrupted x).
        nd_x86_64::sti();
    }

    todo!("Start the `nd_init` process here.");
}
