//! Handcrafted bindings for the Limine boot protocol.
//!
//! <https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md>
//!
//! The Limine boot protocol works by embedding "requests" in the kernel's binary image. The Limine
//! bootloader parses, processes and them responds to them.
//!
//! # Safety
//!
//! This crate assumes that the bootloader responding to requests follows the Limine bootloading
//! protocol.

#![no_std]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

mod bootloader_info;
mod entry_point;
mod memory_map;
mod module;
mod smp;

pub use self::bootloader_info::*;
pub use self::entry_point::*;
pub use self::memory_map::*;
pub use self::module::*;
pub use self::smp::*;

use core::fmt;
use core::mem::MaybeUninit;

/// A pointer which the Limine bootloader will write to.
///
/// Because the modification of this pointer is done outside of Rust (and even before our entry
/// point is called), we need to be careful when attempting to read its value. Specifically, we
/// have to use volatile operations to ensure that the compiler won't attempt to optimize
/// those reads away.
struct ResponsePtr<T>(*mut T);

impl<T> ResponsePtr<T> {
    /// The NULL response pointer.
    pub const NULL: Self = Self(core::ptr::null_mut());

    /// Reads the inner value with volatile semantics.
    #[inline(always)]
    pub fn get(&self) -> *mut T {
        unsafe { core::ptr::read_volatile(&self.0) }
    }
}

impl<T> fmt::Debug for ResponsePtr<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

unsafe impl<T: Sync> Sync for ResponsePtr<T> {}
unsafe impl<T: Send> Send for ResponsePtr<T> {}

/// A limine request.
#[repr(C)]
pub struct Request<Feat: Feature> {
    /// The unique identifier of the request.
    ///
    /// This value is used by the Limine bootloader to detect the request in the kernel image.
    id: [u64; 4],
    /// The revision number of the request.
    ///
    /// The responding bootloader will read this value to determine which version of the response
    /// the kernel expects.
    revision: u64,
    /// The response to the request.
    ///
    /// The Lilmine bootloader will write a valid response instance to this value when they
    /// find themselves
    response: ResponsePtr<Response<Feat>>,
    /// The payload of the request.
    payload: Feat,
}

impl<Feat: Feature> Request<Feat> {
    /// Creates a new [`Request<Feat>`] from the provided request payload.
    pub const fn new(payload: Feat) -> Self {
        let [c, d] = Feat::MAGIC;

        Self {
            id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, c, d],
            revision: Feat::REVISION,
            response: ResponsePtr::NULL,
            payload,
        }
    }

    /// Returns a shared reference to the raw response of this request.
    ///
    /// This is useful if you want to check the revision number of the request yourself.
    #[inline(always)]
    pub fn raw_response(&self) -> Option<&Response<Feat>> {
        unsafe { self.response.get().as_ref() }
    }

    /// Returns an exclusive reference to the raw response object of this request.
    ///
    /// This is useful if you want to check the revision number of the request yourself.
    #[inline(always)]
    pub fn raw_response_mut(&mut self) -> Option<&mut Response<Feat>> {
        unsafe { self.response.get().as_mut() }
    }

    /// Returns a shared reference to the response.
    ///
    /// # Correctness
    ///
    /// This function assumes that *if* Limine has responded to the request, then it must have
    /// provided a valid pointer.
    #[inline(always)]
    pub fn response(&self) -> Option<&Feat::Response> {
        self.raw_response().and_then(Response::payload)
    }

    /// Returns an exclusive reference to the response.
    ///
    /// # Correctness
    ///
    /// This function assumes that *if* Limine has responded to the request, then it must have
    /// provided a valid pointer.
    #[inline(always)]
    pub fn response_mut(&mut self) -> Option<&mut Feat::Response> {
        self.raw_response_mut().and_then(Response::payload_mut)
    }
}

impl<Feat> fmt::Debug for Request<Feat>
where
    Feat: Feature + fmt::Debug,
    Feat::Response: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self.id;

        f.debug_struct("Response")
            .field("id", &format_args!("[{a:x}, {b:x}, {c:x}, {d:x}]"))
            .field("revision", &self.revision)
            .field("response", &self.raw_response())
            .field("payload", &self.payload)
            .finish()
    }
}

/// The response to a Limine [`Request<Feat>`].
#[repr(C)]
pub struct Response<Feat: Feature> {
    /// The revision number of the response.
    revision: u64,
    /// The feature-spesific payload.
    ///
    /// This field is only initialized if `revision` is at least `Feat::EXPECTED_REVISION`
    payload: MaybeUninit<Feat::Response>,
}

impl<Feat: Feature> Response<Feat> {
    /// Returns the revision number of this response.
    #[inline(always)]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns a raw pointer to the payload of this request.
    #[inline(always)]
    pub fn payload_ptr(&self) -> *const Feat::Response {
        self.payload.as_ptr()
    }

    /// Returns a raw pointer to the payload of this request.
    #[inline(always)]
    pub fn payload_mut_ptr(&mut self) -> *mut Feat::Response {
        self.payload.as_mut_ptr()
    }

    /// Returns a shared reference to the payload of this response.
    ///
    /// # Safety
    ///
    /// The `revision` number must be at least `Feat::EXPECTED_REVISION`
    #[inline(always)]
    pub unsafe fn payload_unchecked(&self) -> &Feat::Response {
        self.payload.assume_init_ref()
    }

    /// Returns an exclusive reference to the payload of this response.
    ///
    /// # Safety
    ///
    /// The `revision` number must be at least `Feat::EXPECTED_REVISION`.
    #[inline(always)]
    pub unsafe fn payload_unchecked_mut(&mut self) -> &mut Feat::Response {
        self.payload.assume_init_mut()
    }

    /// Returns a shared reference to the payload of the response.
    ///
    /// If the revision number of the response is not large enough, the function fails by returning
    /// [`None`].
    #[inline(always)]
    pub fn payload(&self) -> Option<&Feat::Response> {
        if self.revision >= Feat::EXPECTED_REVISION {
            Some(unsafe { self.payload_unchecked() })
        } else {
            None
        }
    }

    /// Returns an exclusive reference to the payload of the request.
    ///
    /// If the revision number of the response is not large enough, the function fails by returning
    /// [`None`].
    #[inline(always)]
    pub fn payload_mut(&mut self) -> Option<&mut Feat::Response> {
        if self.revision >= Feat::EXPECTED_REVISION {
            Some(unsafe { self.payload.assume_init_mut() })
        } else {
            None
        }
    }
}

impl<Feat: Feature> Drop for Response<Feat> {
    fn drop(&mut self) {
        if self.revision >= Feat::EXPECTED_REVISION {
            unsafe { self.payload.assume_init_drop() };
        }
    }
}

impl<Feat> fmt::Debug for Response<Feat>
where
    Feat: Feature,
    Feat::Response: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("revision", &self.revision)
            .field("payload", &self.payload())
            .finish()
    }
}

/// A feature supported by the Limine bootloader.
pub trait Feature {
    /// The magic numbers identifying this specific feature.
    ///
    /// The first two magic number are common to every request of the Limine protocol.
    const MAGIC: [u64; 2];
    /// The revision number of the request.
    const REVISION: u64;
    /// The revision number that the response needs to have.
    const EXPECTED_REVISION: u64;
    /// The response type of this request.
    type Response;
}

/// Create a global `.limine_reqs` section with pointers to the provided structs.
#[macro_export]
macro_rules! limine_reqs {
    (
        $(
            $place:expr
        ),*
        $(,)?
    ) => {
        const _: () = {
            #[link_section = ".limine_reqs"]
            #[used(linker)]
            static mut LIMINE_REQS: [*const (); 1 + $crate::limine_reqs!(@ count => $($place,)*)]
                = [
                    $( unsafe { ::core::ptr::addr_of!($place) } as *const (), )*
                    ::core::ptr::null(),
                ];
        };
    };
    ( @ count => $( $place:expr, )* ) => {
        [ $( $crate::limine_reqs!( @ replace => $place, ()) ),* ].len()
    };
    ( @ replace => $place:expr, $($by:tt)* ) => {
        $($by)*
    };
}
