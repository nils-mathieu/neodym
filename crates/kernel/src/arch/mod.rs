//! Architecture-specific code and structures.
//!
//! Depending on the target CPU architecture, resource protection and multiplexing may vastly
//! differ. This module is responsible for two things:
//!
//! 1. Hosting the sub-modules containing the architecture-specific code of the kernel.
//! 2. Providing wrappers for common functionalities required in other (architecture-independent)
//!    parts of the system.

mod x86_64;

pub use self::x86_64::{die, initialize};
