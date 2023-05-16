This tool uses qemu to trace linux programs. Per now, all programs are traced
inside docker, which means you can easily reproduce the exact environment the
executable is supposed to run in (libraries, etc). You can also easily trace
binaries for other platforms by running the container on another architecture.
While qemu lets you run programs from other architectures, it's much easier to
get the correct libraries etc by just emulating the container into the correct
archtecture to begin with.