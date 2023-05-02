# The Neodym Operating System

The present documentation is a **work in progress**. It is not yet complete and some (if not all)
parts of it **will** change in the future.

It provides an overview of the project, its goals, and its design. Implementation details may be
discussed here, but the documentation on a function-by-function basis is provided by the in-code
documentation.

You can build the in-code documentation using `cargo doc --open`.

## What is Neodym?

Neodym is an experimental [exokernel](https://en.wikipedia.org/wiki/Exokernel) and learning
project. I want to learn more about operating system design, and I think that implementing one
is the best way to do so!

Exokernels are a type of operating system that aims to provide as little abstraction to the
hardware as possible without sacrificing security. In other words, they attempt to offload as much
as possible the management of hardware resources to user-space processes without compromising
neither the security nor the performance of the system.

The main inspiration for Neodym is this
[MIT research paper](https://pdos.csail.mit.edu/6.828/2008/readings/engler95exokernel.pdf) by
Dawson R. Engler, M. Frans Kaashoek, and James O'Toole Jr.

## Contents

1. [Booting](booting.md)
