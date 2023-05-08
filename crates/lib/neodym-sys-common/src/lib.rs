//! This libraries defines the types and constants shared between the kernel and userland.

#![no_std]
#![cfg_attr(feature = "try_trait_v2", feature(try_trait_v2))]

use core::fmt;
use core::num::NonZeroUsize;
#[cfg(feature = "try_trait_v2")]
use core::ops::{ControlFlow, FromResidual, Try};

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

/// The return value of a system call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use = "this `SysResult` may be an error, which should be handled"]
#[repr(transparent)]
pub struct SysResult(pub usize);

impl SysResult {
    /// The first value of a [`SysResult`] which can be turned into a [`SysError`] instance.
    ///
    /// This is the smallest allowed value of [`SysError`] without underflowing.
    pub const FIRST_ERROR: usize = (-4096isize) as usize;

    /// Returns whether the inner value of this [`SysResult`] represents an error.
    #[inline(always)]
    pub fn is_error(self) -> bool {
        self.0 >= Self::FIRST_ERROR
    }

    /// Returns whether the inner value of this [`SysResult`] represents success.
    #[inline(always)]
    pub fn is_success(self) -> bool {
        self.0 < Self::FIRST_ERROR
    }

    /// Attempts to convert this [`SysResult`] into a regular [`Result`].
    ///
    /// If the inner value of this [`SysResult`] is greater than or equal to
    /// [`FIRST_ERROR`](SysResult::FIRST_ERROR), then this function will return an [`Err`]
    /// containing a [`SysError`] instance.
    ///
    /// Otherwise, this function will return an [`Ok`] containing the inner value of this
    /// [`SysResult`].
    #[inline(always)]
    pub fn to_result(self) -> Result<usize, SysError> {
        // This match should be optimized away by the compiler as the `ControlFlow` type is
        // designed to be the same as `Result`.
        #[cfg(feature = "try_trait_v2")]
        match core::ops::Try::branch(self) {
            ControlFlow::Continue(output) => Ok(output),
            ControlFlow::Break(residual) => Err(residual),
        }

        #[cfg(not(feature = "try_trait_v2"))]
        match self.0 {
            err @ Self::FIRST_ERROR.. => Err(SysError(err)),
            val => Ok(val),
        }
    }
}

#[cfg(feature = "try_trait_v2")]
impl core::ops::Try for SysResult {
    type Output = usize;
    type Residual = SysError;

    #[inline(always)]
    fn from_output(output: Self::Output) -> Self {
        Self(output)
    }

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            err @ Self::FIRST_ERROR.. => ControlFlow::Break(SysError(err)),
            val => ControlFlow::Continue(val),
        }
    }
}

#[cfg(feature = "try_trait_v2")]
impl core::ops::FromResidual for SysResult {
    fn from_residual(residual: <Self as core::ops::Try>::Residual) -> Self {
        Self(residual.0)
    }
}

/// An error which might be encoded in a [`SysResult`] instance.
///
/// # Representation
///
/// It is important to note that the "valid" values of this type do not start at zero, but at
/// [`SysResult::FIRST_ERROR`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SysError(pub usize);

/// This macro defines the constants of the [`SysError`] type, generating some additional functions
/// for working with them.
macro_rules! define_SysError_constants {
    (
        $(
            #[ doc = $doc:expr ]
            pub const $name:ident = $offset:expr;
        )*
    ) => {
        impl SysError {
            $(
                #[ doc = $doc ]
                pub const $name: Self = Self(SysResult::FIRST_ERROR + $offset);
            )*

            /// Returns the name of this error, as a static string.
            pub const fn name(self) -> &'static str {
                match self {
                    $(
                        Self::$name => stringify!($name),
                    )*
                    _ => "UNKNOWN",
                }
            }

            /// Returns a short description of this error, as a static string.
            pub const fn description(self) -> &'static str {
                match self {
                    $(
                        Self::$name => $doc,
                    )*
                    _ => "Unknown Error",
                }
            }
        }
    };
}

define_SysError_constants! {}

impl fmt::Debug for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SysError").field(&self.name()).finish()
    }
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.description())
    }
}

/// A unique identifier for a process in the system.
pub type ProcessHandle = NonZeroUsize;
