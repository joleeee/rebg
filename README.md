This tool uses qemu to trace linux programs. Per now, all programs are traced
inside docker, which means you can easily reproduce the exact environment the
executable is supposed to run in (libraries, etc). You can also easily trace
binaries for other platforms by running the container on another architecture.
While qemu lets you run programs from other architectures, by using qemu inside
docker (qemu), it's much easier to get the correct libraries as you can just
install them inside the container.

# Performance on MacOS
While performance is not too big of a worry when tracing, it's especially nice
to have a speedup when building our patched qemu. If you're using macos you can
decrease your crossplatform buildtime by about 60% (2.4x speedup) by using
rosetta with docker. In Docker Desktop you can find it under "Features in
development". For me the speedup was as following on a Macbook Air M2, 10GB of
ram and 7 cores allocated to docker. There was proably a lot of thermal
throttling.

```
== rosetta ==
./configure: 77.7s
make -j8: 2491.4s

== default (qemu) ==
./configure: 188.8s
make -j8:

== native (macos) ==
```