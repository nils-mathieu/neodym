# Installation Guide

No pre-compiled binaries are provided for the Neodym kernel at the moment. You'll need to build it
from source.

## Building From Source

In order to build the project from source, you will need the following programs:

- A working Rust toolchain (e.g. `cargo` and `rustc`)
- Standard development utilities, such as `git`, `make`, `cp`...

### Rust

Neodym is written in Rust. You'll need to latest Rust nightly toolchain in order to compile it.

Specifically, you will need the following components: `cargo`, `rustc`.

You can get both on [Rust's official website](https://www.rust-lang.org/learn/get-started).

### Compilation

```bash
cargo build --release --package neodym --target targets/x86_64.json
cargo build --release --package nd_init --target targets/x86_64.json
```

By default, the final binaries will be located in `target/x86_64/release/`.

Rust can be used as a cross-compiler by passing a `--target` to Cargo. At the moment, Neodym only
supports **x86_64** CPUs, so you'll need to use `--target targets/x86_64.json`.

`--release` indicates that we're using the "release" profile, which enables aggressive
optimizations and removes some sanity checks.

## Creating An Bootable Image

In order to create a bootable image, we need three things:

1. A bootloader supported by Neodym.
2. The Neodym kernel.
3. The `nd_init` program.

### Booting With Limine

The only supported bootloader is [Limine](https://github.com/limine-bootloader/limine), at the
moment.

You can grab the latest version of the bootloader from their repository.

```bash
git clone https://github.com/limine-bootloader/limine.git \
    --branch=v4.x-branch-binary \
    --depth=1 \
    limine-repos
make -C limine-repos
```

### Creating an ISO

After that, you'll need to create a directory that will hold the `.iso` image.

With the limine bootloader, it will look like this:

```txt
iso_root/
    limine-cd-efi.bin
    limine-cd.bin
    limine.sys
    limine.cfg
    nd_init
    neodym
```

`limine-cd-efi.bin`, `limine-cd.bin` and `limine-sys.bin` are all present in the `limine-repos`
directory created earlier.

`limine.cfg` is the configuration file of Limine. If you want to multiboot Neodym without another
operating system, you can use this configuration file.

Here is an example `limine.cfg` configuration that works with the above directory structure:

```txt
:Neodym
    COMMENT=The Neodym Operating System
    PROTOCOL=limine
    KERNEL_PATH=boot:///neodym
    MODULE_PATH=boot:///nd_init
```

To turn the directory into an ISO image, you can use an utility like `xorriso`:

```bash
xorriso \
    -as mkisofs \
    -b limine-cd.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-in fo-table \
    --efi-boot limine-cd-efi.bin \
    -efi-boot-part \
    --efi-boot-image \
    --protectiv e-msdos-label \
    iso_root \
    -o image.iso
```

You still have to install the Limine bootloader on the image's _Master Boot Record_ with the
`limine-deploy` command provided by the Limine repository.

```bash
limine-repos/limine-deploy image.iso
```

## Booting With Qemu

For the **x86_64** architecture, you can use the following commands.

Either with graphics:

```bash
qemu-system-x86_64 \
    -m 2G \ # 2GB of RAM
    -cdrom image.iso \
    -boot d \
    -no-reboot \
    -serial stdio
```

Or without.

```bash
qemu-system-x86_64 \
    -m 2G \ # 2GB of RAM
    -cdrom image.iso \
    -boot d \
    -no-reboot \
    -nographic
```
