//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.
//!

use nd_limine::File;

use crate::sys_info::{SysInfo, SysInfoTok};

mod req;

/// Removes the begining of a path, only keeping the what's after the last `/` character.
fn get_filename(bytes: &[u8]) -> &[u8] {
    let start_idx = match bytes.iter().rposition(|&b| b == b'/') {
        Some(slash) => slash + 1,
        None => 0,
    };

    unsafe { bytes.get_unchecked(start_idx..) }
}

/// Reads The content of the [`MODULE`] request and returns the file that has been loaded.
///
/// # Panics
///
/// If the init program is not present, this function panics with an appropriate error message.
fn find_init_program() -> Option<&'static File> {
    nd_log::trace!("Enumerating kernel modules...");

    let response = req::MODULE.response()?;

    let mut found = None;

    for module in response.modules().iter().filter_map(|x| x.file()) {
        nd_log::trace!(" - {:?}", module.path());

        // We're looking for a file named 'nd_init'.
        if get_filename(module.path().to_bytes()) == b"nd_init" {
            found = Some(module);
        }
    }

    found
}

const KERNEL_STACK_SIZE: usize = 4096 * 16;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

/// The entry point of the kernel when booted by the Limine bootloader on **x86_64**.
#[naked]
extern "C" fn entry_point() -> ! {
    unsafe {
        // We need to setup a stack within the kernel's address space, as the bootloader's memory
        // eventually gets reclaimed as usable memory.
        //
        // NOTE:
        //  We can use JMP instead of a regular CALL as the function won't return anyway.
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

extern "C" fn entry_point_inner() -> ! {
    // SAFETY:
    //  We're in the entry point, this function won't be called ever again.
    unsafe { crate::logger::initialize() };

    //
    // Gather the responses from the Limine bootloader.
    // Some are necessary, others are just nice information to have.
    //

    if let Some(info) = req::BOOTLOADER_INFO.response() {
        nd_log::info!("Loaded by '{}' (v{})!", info.name(), info.version());
    } else {
        nd_log::info!("Loaded by a Limine-compliant bootloader.");
    }

    if req::ENTRY_POINT.response().is_none() {
        nd_log::warn!("The Limine bootloader did not respond to the entry point request.");
        nd_log::warn!("  > This is just a sanity check.");
        nd_log::warn!("  > The bootloader might be corrupted.");
    }

    let Some(kernel_addr) = req::KERNEL_ADDR.response() else {
        nd_log::error!("The Limine bootloader did not provide the address of the kernel.");
        crate::die();
    };

    let Some(hhdm) = req::HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM offset.");
        crate::die();
    };

    let Some(_memmap) = req::MEMORY_MAP.response() else {
        nd_log::error!("The Limine bootloader did not provide a map of the usable memory.");
        crate::die();
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
        crate::die();
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
        crate::tables::setup_gdt();
        crate::tables::setup_idt();
        crate::tables::setup_system_calls();
        crate::apic::initialize_lapic();

        // Enable interrupts. We're ready to be interrupted x).
        nd_x86_64::sti();
    }

    todo!("Start the `nd_init` process here.");
}
