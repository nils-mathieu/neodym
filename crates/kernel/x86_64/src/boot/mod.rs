//! Code specific to the boot sequence of the kernel.
//!
//! # Bootloader Support
//!
//! At the moment, only the [Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md)
//! protocol is supported, under the [`limine`] module.

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod limine;
