# The Neodym Operating System

This repository is home of the Neodym Operating System. Neodym is an experimental
[exokernel](https://en.wikipedia.org/wiki/Exokernel) and learning project.

## State Of The Project

Nothing to see yet! The project is still in its early stages of development.

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

Neodym uses way to many open-source projects, papers, blog posts and other resources to list them
all in this README. A complete list is available [here](ACKNOWLEDGEMENTS.md).
