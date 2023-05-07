//! A simple logging facade for the Neodym Operating System.

#![no_std]
#![warn(missing_docs, missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::fmt::Arguments;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::Relaxed;

/// A verbosity level associated with a [`Record`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
pub enum Verbosity {
    Error,
    Warn,
    Info,
    Trace,
}

/// A record that can be logged by the global logger.
#[derive(Debug, Clone, Copy)]
pub struct Record<'a> {
    /// The verbosity level of this record.
    pub verbosity: Verbosity,
    /// The message.
    pub message: Arguments<'a>,
    /// The file from which the record originates.
    pub file: &'static str,
    /// The line within the file from which this record originates.
    pub line: u32,
}

/// Creates a [`Record`] for the current call-site.
#[macro_export]
macro_rules! record {
    ($verbosity:expr, $($args:tt)*) => {
        $crate::Record {
            verbosity: $verbosity,
            message: ::core::format_args!($($args)*),
            file: ::core::file!(),
            line: ::core::line!(),
        }
    };
}

/// The signature of the function that will be called when a [`Record`] needs to be logged.
pub type LoggerFn = fn(record: &Record);

/// The default logging function.
fn noop_logger(_record: &Record) {}

/// An atomic [`LoggerFn`] which is used to log messages.
static GLOBAL_LOGGER: AtomicPtr<u8> = AtomicPtr::new(noop_logger as *mut u8);

/// Sets the global logging function which should be used when receiving [`Record`]s.
#[inline(always)]
pub fn set_global_logger(f: LoggerFn) {
    GLOBAL_LOGGER.store(f as *mut u8, Relaxed);
}

/// Removes the global logger.
#[inline(always)]
pub fn remove_global_logger() {
    set_global_logger(noop_logger);
}

/// Loads the current global logging function.
#[inline(always)]
pub fn get_global_logger() -> LoggerFn {
    let p = GLOBAL_LOGGER.load(Relaxed);

    // SAFETY:
    //  We know by invariant of `GLOBAL_LOGGER` that it always contain a valid `LoggerFn` pointer.
    unsafe { core::mem::transmute(p) }
}

/// Logs a message using the global logger.
#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($args:tt)*) => {
        $crate::get_global_logger()(&$crate::record!($verbosity, $($args)*))
    };
}

/// Logs a message with the [`Verbosity::Error`] level.
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Error, $($args)*);
    };
}

/// Logs a message with the [`Verbosity::Warn`] level.
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Warn, $($args)*);
    };
}

/// Logs a message with the [`Verbosity::Info`] level.
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Info, $($args)*);
    };
}

/// Logs a message with the [`Verbosity::Trace`] level.
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Trace, $($args)*);
    };
}
