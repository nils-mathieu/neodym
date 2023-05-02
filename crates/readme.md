# Crates Overview

This directory contains all the crates that make up the Neodym project.

- `arch/` contains architecture-specific abstractions. For example, the `arch/x86_64` crate
  provides structures and functions that are specific to the x86_64 architecture, but not to the
  Neodym kernel itself.
- `boot/` contains implementation of bootloading protocols.
- `utility/` contains utility crates, such as `nd_log`, which is used to gather log messages
  within the kernel.
- `kernel` is the kernel itself.
- `nd_init` is the first process that the kernel starts. This will probably move to a separate
  repository at some point.
