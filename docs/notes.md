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

## Sheduling

The scheduler manages time slices "quantums" which can be allocated by processes.

A process can allocate a quantum by calling the `nd::sched::allocate` system call. It will then
be scheduled to run for the duration of the quantum by the scheduler.

A process may donate the rest of its quantum to another process (potentially specific) by calling
the `nd::sched::yield` system call.

## Physical Memory

Physical memory is allocated and deallocate with the `nd::mem::insert` system call. This system
call allows a process to insert a page table entry. Deallocation simply consists of inserting a
page table entry with a cleared `PRESENT` flag.

Shared pages can be inserted by mapping an already mapped page. This requires a special capability
with the processes sharing the page.
