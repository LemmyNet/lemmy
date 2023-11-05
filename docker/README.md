# Building Lemmy Images

## Introduction

Lemmy images are of two kind:
- `builder` (Including rust and toolchains to compile the project)
- `runner` (Lightweight image that will run on the final users' infrastructure)

## Constraints

### Multi-architecture

We have multi-architecture constraints to build and distribute Lemmy.
Users may run Lemmy on AMD64 or ARM64, contributors may build on ARM64
(e.g. if you have an Apple Silicon)...

The Lemmy project has a strong constraint as of this writing:
We use GitHub-hosted runners that are **amd64** only.

Which means our official CD and release pipeline can only use `amd64 builder` images.
And that our compilation performance are pretty bad as GitHub runners are
low-end machine.

It also means we need a `aarch64-unknown-linux-gnu` toolchain that will be responsible
for linking Lemmy *code objects* and the shared libraries when running on an
**amd64** platform.

### Shared Libraries

The project uses two shared libraries:
- `libssl-dev`
- `pg`

Where I (@raskyld) got confused the most (and where most readers uninformed on
how the linker process work will get too) is that we may believe because
they are shared libraries they are not needed when we build the project.

This is true at **compile-time** but it *may* not be at **linking-time**!

When it comes to shared libraries, the resolution of dependencies can be mixed
between **run-time** (on the user infrastructure) and the **linking-time**, which
happens after `rustc` compiled our code.

> Disclaimer: The following part is my own interpretation and may be incorrect.

On Unix systems, it seems common to prefer **linking-time** over of **run-time** for
optimization and performance reason. The **run-time** resolution can be interpreted as
a **plugin** system in the sense that our program will contains code responsible for
loading shared libraries we have absolutely no knowledge of.

This is done by invoking [`ld-linux`][ld-linux-man] which is not done by hand but is the
result of using the C library [`dlopen()`][dlopen-man] provided by `glibc` or `musl`.

> End of Disclaimer

In our specific case, all our dependencies are linked at **linking-time** this means
the *libraries objects* need to be present in the builder context and they need to
be compiled for the targeted architecture.

#### References

- [The Linux Documentation Project on Shared Libraries][tldp-lib]

[tldp-lib]: https://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html
[ld-linux-man]: https://linux.die.net/man/8/ld-linux
[dlopen-man]: https://linux.die.net/man/3/dlopen
