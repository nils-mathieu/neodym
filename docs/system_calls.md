# System Calls

Because system call mechanisms are very much architecture dependent, and because the way the
hardware is accessed and manipulated changes from one architecture to another, the system call
interface changes from one to another.

## Type Definitions

```c
typedef uint64_t ProcessHandle;
```

## x86_64

On x86_64, system calls are performed using the `syscall` instruction. The system call number is
passed in the `rax` register, and the arguments are passed to `rdi`, `rsi` and `rdx`.

The return value is stored in `rax`.

| Mnemonic                | `rax` | `rdi`   | `rsi` | `rdx` |
| ----------------------- | ----- | ------- | ----- | ----- |
| [Terminate](#Terminate) | 0     | process |       |       |
| [MapMemory](#MapMemory) | 1     | process | pages | count |

### Terminate

```c
void terminate(ProcessHandle process);
```

The `Terminate` system call terminates the specified process.

If the provided parameter is `0`, the current process is terminated and the system call never
returns control to the caller.

#### Permissions

To terminate another process, the process must have the `Terminate` permission over the target
process. This permission is not required when terminating the current process.

#### Errors

- `PermissionDenied` is returned if the caller does not have the necessary permissions to terminate
  the specified process.

### MapMemory

```c
#define MAP_MEMORY_READABLE   (1 << 0)
#define MAP_MEMORY_WRITABLE   (1 << 1)
#define MAP_MEMORY_EXECUTABLE (1 << 2)
#define MAP_MEMORY_UNMAP      (0 << 3)
#define MAP_MEMORY_SIZE_4K    (1 << 3)
#define MAP_MEMORY_SIZE_2M    (2 << 3)
#define MAP_MEMORY_SIZE_2G    (3 << 3)

SysResult map_memory(ProcessHandle proces, uint64_t const *entries, size_t count);
```

The `MapMemory` system call maps or unmaps a physical page of memory to the virtual address space
of the target process.

`process` is the handle of the process to map the memory to. When `process` is `0`, the current
process is used.

`entries` is a pointer to an array of `count` mapping entries.

Each entry is separated into two parts:

```
             57                                  12                0
+--------------+-----------------------------------+----------------+
| 7 bits count | 45 bits address                   | 12 bits flags  |
+--------------+-----------------------------------+----------------+
```

The flag bits are used to describe the requested mapping.

#### Permissions

Attempting to map the memory of another process requires the `MapMemoryOf` permission over the
target process. This permission is not required when mapping the memory of the current process.

#### Mapping Size

Pages can be mapped in three different sizes:

- `MAP_MEMORY_SIZE_4K` indicates that the page is four KiB in size.
- `MAP_MEMORY_SIZE_2M` indicates that the page is two MiB in size.
- `MAP_MEMORY_SIZE_2G` indicates that the page is two GiB in size.

When none of those flags are set (`MAP_MEMORY_UNMAP`), the page is unmapped instead.

When the mapping is already present, attempting to change its size will result in an error.

#### Protection Flags

Pages can have specific protections applied to them.

- `MAP_MEMORY_READABLE` indicates that the page can be read from.
- `MAP_MEMORY_WRITABLE` indicates that the page can be written to.
- `MAP_MEMORY_EXECUTABLE` indicates that the page can be executed.

Note that those protection are only applied to the target process.

If the mapping is already present, the protection flags are simply updated to the requested values.

#### Page Count

The `count` field in each entry indicates the number of adjacent pages to map. When multiple pages
are mapped to the same virtual address, the last page is the one that's actually mapped when they
are compatible.

If two entries are not compatible (i.e. they have different size for the same address), the system
call will return `InvalidParameter` and no mapping will be performed.

#### Returns

This function always returns `0` on success.

#### Errors

- `InvalidParameter` if one of the provided input parameters is invalid.

  - Attempting to pass an invalid pointer as the `pages` parameter.
  - Attempting to map a misaligned address.
  - Attempting to unmap a misaligned address, or absent mapping.
  - Providing two incompatible entries.

- `PermissionDenied` if the current process does not have the required permissions.

- `OutOfMemory` if there is no more physical memory available.

  - It is also possible for this error to be returned if the process allocated to much memory and
    the kernel prevents it from allocating more.
