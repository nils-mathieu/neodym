//! The entry point of the kernel, when booted by the
//! [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) bootloader.
//!
//! Because the Limine bootloader supports two different architectures, their respective entry
//! points are defined in the [`x86_64`] and `aarch64` (not yet implemented) modules.

use nd_limine::limine_reqs;
use nd_limine::{
    BootloaderInfo, EntryPoint, File, Hhdm, KernelAddress, MemoryMap, Module, Request,
};

use crate::sys_info::{SysInfo, SysInfoTok};

/// Requests the bootloader to provide information about itself, such as its name and version.
/// Those information will be logged at startup.
static BOOTLOADER_INFO: Request<BootloaderInfo> = Request::new(BootloaderInfo);

/// Requests the Limine bootloader to call a specific function rather than the entry point specified
/// in the ELF header.
static ENTRY_POINT: Request<EntryPoint> = Request::new(EntryPoint(entry_point));

/// Requests Limine to load an additional module along with the kernel itself.
///
/// This module will contain the initial program to start after the kernel has initialize itself.
static MODULE: Request<Module> = Request::new(Module::new(&[]));

/// Requests the Limine bootloader to provide a map of the available physical memory.
static MEMORY_MAP: Request<MemoryMap> = Request::new(MemoryMap);

/// Requests the Limine bootloader to provide the address of the kernel in physical memory.
static KERNEL_ADDR: Request<KernelAddress> = Request::new(KernelAddress);

/// The Limine bootloader maps the entierty of the physical memory to the higher half of the
/// virtual address space.
///
/// This request provides the address of th *Higher Half Direct Map* offset.
static HHDM: Request<Hhdm> = Request::new(Hhdm);

limine_reqs!(
    MEMORY_MAP,
    BOOTLOADER_INFO,
    MODULE,
    ENTRY_POINT,
    HHDM,
    KERNEL_ADDR
);

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

    let response = MODULE.response()?;

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
    unsafe { crate::logger::initialize() };

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
        crate::die();
    };

    let Some(hhdm) = HHDM.response() else {
        nd_log::error!("The Limine bootloader did not provide the HHDM offset.");
        crate::die();
    };

    let Some(_memmap) = MEMORY_MAP.response() else {
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
