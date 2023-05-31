# What?
Traces programs, so you can debug them without breakpoints. Breakpoints are
annoying.

Logs all ran cpu instructions, register writes, memory writes, and syscalls.

Per now it only works with linux program, both `x86_64` and `aarch64`. Programs
are traced inside docker, so you can trace them from any operating system. This
also means you can trace different architectures, because docker supports
emulation through qemu. This also makes a lot of stuff work out of the libc,
wrt. libc and other libraries. It does mean you have up to two layers of
emulation.

# Performance on MacOS
If you're using macos and tracing `linux/amd64`, you can cut runtime by about
60% by using rosetta with docker. In Docker Desktop you can find it under
"Features in development". 

# Future work
- PIN: Linux & Windows tracing.
- Web ui
- Efficently storing memory history.