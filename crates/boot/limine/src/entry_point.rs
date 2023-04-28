use crate::Feature;

/// The signature of the function that Limine assumes to call when responding to an entry point
/// request.
pub type EntryPointFn = extern "C" fn() -> !;

/// Requests Limine to call a specific function as the entry point of the kernel.
///
/// By default, if this requeste is not specified, Limine calls the entry point specified in the
/// ELF header of the kernel image.
#[derive(Debug)]
pub struct EntryPointRequest(pub EntryPointFn);

/// The response to the [`EntryPointRequest`].
#[derive(Debug)]
pub struct EntryPointResponse;

impl Feature for EntryPointRequest {
    const MAGIC: [u64; 2] = [0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a];
    const REVISION: u64 = 0;
    const EXPECTED_REVISION: u64 = 0;
    type Response = EntryPointResponse;
}
