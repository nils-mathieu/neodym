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

| Mnemonic                | `rax` | `rdi`    | `rsi` | `rdx` |
| ----------------------- | ----- | -------- | ----- | ----- |
| [Terminate](#Terminate) | 0     | process  |       |       |
| [GetMemory](#GetMemory) | 1     | segments | count |       |

### Terminate

```c
void terminate(ProcessHandle process);
```

The `Terminate` system call terminates the specified process.

If the provided parameter is `0`, the current process is terminated and the system call never
returns control to the caller.

#### Returns

This system call always returns `0` on success, or never if the specified process is the current
one.

#### Permissions

To terminate another process, the process must have the `Terminate` permission over the target
process. This permission is not required when terminating the current process.

#### Errors

- `PermissionDenied` is returned if the caller does not have the necessary permissions to terminate
  the specified process.

### GetMemory

```c
typedef struct {
    start: uint64_t;
    size: uint64_t;
} Segment;

SysResult get_memory(Segment *segments, size_t count);
```

The `GetMemory` system call is used to retrieve the memory segments currently available on the
system.

`count` is the number of `Segment` structures that can be stored in the buffer referenced by
`segments`.

The kernel will write at most `count` instances of `Segment` to the buffer referenced by `segments`.

Those segments are guarenteed to be non-overlapping, and sorted by ascending order of their
addresses.

#### Returns

This system call always succeeds and returns the number of segments available on the system.

Note that this _does not_ depend on the provided `count` value. This system call will always write
at most `count` elements to the `segments` buffer.

#### Errors

This system call never fails.
