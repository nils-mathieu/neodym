mod page_based_allocator;
mod page_box;

pub mod page_list;

pub use self::page_based_allocator::*;
pub use self::page_box::*;
pub use self::page_list::PageList;

/// The size of a single page.
#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE: usize = 4096;
