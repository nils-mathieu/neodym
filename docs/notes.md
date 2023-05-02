# Notes

This document simply contains notes about the project, and how I plan to implement certain
features.

## Permissions

The kernel keeps track of the permissions of each process. Specifically, it keeps track of
which processes are allowed to access which resources.

The Idea is that a process shouldn't be able to access a resource unless it has been
explicitly granted access to it. Only a process with a specifial permission can grant access
to a resource.

Example permissions:

- Grant permissions to other processes (only possible for permissions that the process itself has).
- Read from unowned memory (otherwise, only owned memory can be read from).
- Write to unowned memory (otherwise, only owned memory can be written to).
- Spawn a process.
- Access PCI devices, USB devices, etc.
- Access the network, request DNS resolution, read all packets, etc.
- Read from the physical disk
- Write to the physical disk
