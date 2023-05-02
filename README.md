# The Neodym Operating System

This repository is home of the Neodym Operating System. Neodym is an experimental
[exokernel](https://en.wikipedia.org/wiki/Exokernel) and learning project.

## Hardware Support

The kernel aims to support multiple CPU architecture (though it is currently mainly focused on
`x86_64`).

Similarly, support for multiple bootloading protocol is planned, though currently, only
[Limine](https://github.com/limine-bootloader/limine/blob/v4.x-branch/PROTOCOL.md) is supported.

Currently, the kernel has only been tested inside of the [QEMU](https://www.qemu.org/) emulator.

## Documentation

The in-code documentation of the project can be generated using `cargo doc --open`.

Some additional documentation (such as design documents and overviews) can be found in the
[docs](docs) directory.

## Acknowledgements

- [OSDev.org](https://wiki.osdev.org/Expanded_Main_Page) has been really helpful to get me started.
- The [`x86_64`](https://crates.io/crates/x86_64) crate (though not directly used in this project)
  has been used as reference when implementing some of the x86_64-specific logic.
- This [MIT research paper](https://pdos.csail.mit.edu/6.828/2008/readings/engler95exokernel.pdf)
  on Exokernel design by Dawson R. Engler, M. Frans Kaashoek, and James O'Toole Jr.
