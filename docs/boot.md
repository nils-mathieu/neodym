# Booting

## Bootloader

Neodym does not implement its own bootloader. Instead, it relies on bootloading protocols provided
by other projects.

There is two main reasons for this:

1. Creating a good bootloader is pretty tricky and time-consuming. I'd rather focus on kernel
   design.
2. Using an already established bootloader allows Neodym to be used in a multi-boot environment,
   alongside other operating systems.

### Limine

The [Limine](https://github.com/limine-bootloader/limine/tree/trunk) bootloader loads the kernel in
64-bit long mode, with paging already enabled. The kernel simply has to setup the **GDT** and the
**IDT**, before loading the **nd_init** process.

The **nd_init** process is expected to be loaded as a kernel module named "nd_init".

Example `limine.cfg`:

```text
:Neodym
    COMMENT=The Neodym Operating System
    PROTOCOL=limine
    KERNEL_PATH=boot:///neodym
    MODULE_PATH=boot:///nd_init
```

## nd_init

**nd_init** is the name of the first process that the kernel loads and starts. It should never
actually return control to the kernel, and should instead spawn whatever processes are needed to
start the system.

Because the kernel does not include the concept of filesystem (and that's by design!), it cannot
really find a **nd_init** file by itself. Instead, it relies on the bootloader to load **nd_init**.

**nd_init** is a flat binary (i.e. not an ELF binary). The specifics of how it is loaded by the
kernel are not yet defined.

A flat binary can be created from a regular ELF binary using the `objcopy` utility:

```bash
objcopy --output-target=binary nd_init.elf nd_init
```

Depending on how the kernel will be used (i.e. as a command-line server, or as a graphical desktop),
**nd_init** will have different responsibilities. For example, if the kernel is used as a graphical
desktop, **nd_init** will start the window manager and the desktop environment.

**nd_init** is always trusted by the kernel. It has all possible privileges.
