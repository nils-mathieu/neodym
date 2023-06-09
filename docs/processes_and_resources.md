# Processes And Resources

An operating system is literally an implementation of the concept of a process. Physical resources
are limited, and creating an operating system is implementing a way to share those resources between
mutually distrusting processes.

## Processes

On Neodym, a process is simply an actor with a set of permissions over the resources available on
the system. Processes can enforce their rights over resources using the system call interface. When
a system call is issued, the kernel checks whether the process has the necessary permissions to
perform the requested operation. If it does, the system call completes. Otherwise, an error is
returned to the process indicating that it does not have the necessary permissions.

## Resources

Resources are literal and concrete things available on the system. Physical RAM, CPU time, disk
space, etc. However, files, sockets, and other abstractions are _not_ resources. They are simply
abstractions over resources which processes can use if they wish to.
