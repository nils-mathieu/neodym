use nd_limine::{BootloaderInfo, EntryPoint, KernelAddress, MemoryMap, Module, Request};

/// Requests the bootloader to provide information about itself, such as its name and version.
/// Those information will be logged at startup.
pub static BOOTLOADER_INFO: Request<BootloaderInfo> = Request::new(BootloaderInfo);

/// Requests the Limine bootloader to call a specific function rather than the entry point specified
/// in the ELF header.
pub static ENTRY_POINT: Request<EntryPoint> = Request::new(EntryPoint(super::entry_point));

/// Requests Limine to load an additional module along with the kernel itself.
///
/// This module will contain the initial program to start after the kernel has initialize itself.
pub static MODULE: Request<Module> = Request::new(Module::new(&[]));

/// Requests the Limine bootloader to provide a map of the available physical memory.
pub static MEMORY_MAP: Request<MemoryMap> = Request::new(MemoryMap);

/// Requests the Limine bootloader to provide the address of the kernel in physical memory.
pub static KERNEL_ADDR: Request<KernelAddress> = Request::new(KernelAddress);

nd_limine::limine_reqs!(
    MEMORY_MAP,
    BOOTLOADER_INFO,
    MODULE,
    ENTRY_POINT,
    KERNEL_ADDR
);
