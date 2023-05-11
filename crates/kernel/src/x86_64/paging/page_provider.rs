use core::fmt;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use nd_x86_64::PhysAddr;

use super::{MemorySegment, OutOfPhysicalMemory};

/// Returns a [`fmt::Debug`] implementation that displays the given number of bytes in a human
/// readable format.
fn human_bytes(bytes: u64) -> impl fmt::Display {
    struct Bytes(u64);

    impl fmt::Display for Bytes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut bytes = self.0;

            let mut write_dec =
                |n: u64, dim: &str| write!(f, "{}.{} {}", n / 1024, ((n % 1024) * 100) / 1024, dim);

            if bytes < 1024 {
                return write!(f, "{} B", bytes);
            }

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "KiB");
            }

            bytes /= 1024;

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "MiB");
            }

            bytes /= 1024;

            if bytes < 1024 * 1024 {
                return write_dec(bytes, "GiB");
            }

            bytes /= 1024;

            // wtf so much memory
            write_dec(bytes, "TiB")
        }
    }

    Bytes(bytes)
}

/// Provides a stream of physical pages.
///
/// Note that this type does not provide any way to free those pages.
pub struct PageProvider {
    segments: nd_array::Vec<MemorySegment, { Self::MAX_SEGMENTS }>,
    index: AtomicUsize,
}

impl PageProvider {
    /// The maximum number of segments that can be iterated over.
    const MAX_SEGMENTS: usize = 16;

    /// Creates a new [`PageIterator`] instance.
    pub fn new(usable: &mut dyn Iterator<Item = MemorySegment>) -> Self {
        let mut segments = nd_array::Vec::<MemorySegment, { Self::MAX_SEGMENTS }>::new();
        let mut pages = 0;
        for segment in usable.take(segments.capacity()) {
            pages += segment.length / 0x1000;

            if let Some(last) = segments.last_mut() {
                // Attempt to merge the current segment with the last one.
                if last.base + last.length == segment.base {
                    last.length += segment.length;
                    continue;
                }
            }

            unsafe { segments.push_unchecked(segment) };
        }

        let remaining = usable.count();
        if remaining != 0 {
            nd_log::warn!("Too many usable memory regions, {remaining} have been ignored.");
        }

        nd_log::info!(
            "{} pages of usable memory, in {} contiguous segments, {} in total.",
            pages,
            segments.len(),
            human_bytes(pages * 0x1000)
        );

        Self {
            segments,
            index: AtomicUsize::new(0),
        }
    }

    /// Allocates a single page.
    pub fn allocate(&self) -> Result<PhysAddr, OutOfPhysicalMemory> {
        // The index of the page that will be allocated.
        //
        // Relaxed ordering is sufficient here because we only care about the order of the
        // operations on this specific atomic variable. If another threads attempts to allocate
        // a page, their operation will be ordered with respect to this one, and we don't really
        // care which happens before or after the other.
        let mut page_index = self.index.fetch_add(1, Relaxed) as u64;

        // This executes in O(n), with n being the number of segments.
        // This is fine, as we don't expect to have more than `MAX_SEGMENT_COUNT` segments. It will
        // usually be 4 to 8 segments.
        for segment in &self.segments {
            let page_count = segment.length / 4096;

            if page_index < page_count {
                // We found the right segment for the index!
                return Ok(segment.base + page_index * 4096);
            }

            page_index -= page_count;
            // not in this segment
        }

        // We need to restore the previous index in order to prevent the index from overflowing.
        // If `next_free` overflows, then used segments will start being allocated again. This is
        // actually pretty bad, but there's not much we can do about it without using a lock.
        //
        // This races with the `fetch_add` above, but if other threads are able to allocate enough
        // pages to overflow an `usize` by the time we get here, then the system is probably having
        // bigger issues than this.
        //
        // I think locking would actually be fine, but it's so unlikely that this will be an issue
        // that the lock-free implementation is probably worth it.
        self.index.store(page_index as usize, Relaxed);

        // We're out of memory :(
        Err(OutOfPhysicalMemory)
    }
}
