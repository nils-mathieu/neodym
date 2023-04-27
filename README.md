# The Neodym Operating System

This repository is home of the Neodym Operating System. Neodym is an experimental
[exokernel](https://en.wikipedia.org/wiki/Exokernel) and learning project.

## Hardware Support

The kernel aims to support multiple CPU architecture (though it is currently mainly focused on
`x86_64`).

Similarly, support for multiple bootloading protocol is planned, though currently, only
[Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) is supported.

Currently, the kernel has only been tested inside of the [QEMU](https://www.qemu.org/) emulator.