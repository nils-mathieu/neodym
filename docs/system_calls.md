# System Calls

Because system call mechanisms are very much architecture dependent, and because the way the
hardware is accessed and manipulated changes from one architecture to another, the system call
interface changes from one to another.

## x86_64

On x86_64, system calls are performed using the `syscall` instruction. The system call number is
passed in the `rax` register, and the arguments are passed to `rdi`, `rsi` and `rdx`.

The return value is stored in `rax`.

| Mnemonic | `rax` |
| -------- | ----- |
