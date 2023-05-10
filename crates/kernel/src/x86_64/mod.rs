mod boot;

mod apic;
mod interrupts;
mod logger;
mod sys_info;
mod tables;

pub use self::apic::*;
pub use self::interrupts::*;
pub use self::logger::*;
pub use self::sys_info::*;
pub use self::tables::*;

/// Returns the size of the kernel image, in bytes.
fn image_size() -> usize {
    // This is a symbol defined in the linker script. Its *address* will be defined to the size of
    // unpacked kernel image.
    extern "C" {
        #[link_name = "__nd_image_size"]
        static IMAGE_SIZE: u8;
    }

    // SAFETY:
    //  This static external variable is set by the linker script, and won't change afterwards.
    unsafe { &IMAGE_SIZE as *const u8 as usize }
}
