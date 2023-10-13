# What?
Traces programs, so you can debug them without breakpoints. Breakpoints are
annoying.

Logs all ran cpu instructions, register writes, memory writes, and syscalls.

Per now it only works with linux programs, both `x86_64` and `aarch64`. Programs
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

# UI
We have a UI now! :D

You can use basic vim bindings: `j/k` for down/up. I'm not sure if we want to
override the arrow keys, but we'll probably do it eventually.

Recording all state allows to introduce some bindings which look like things you may be familiar with from gdb:

## next / last instruction
- `Shift-J`: equivalent to gdb `ni`
- `Shift-K`: the same, but in reverse (time-wise)

## step to next call / parent
- `h`: step to whatever called this (back in time)
- `l`: step to the next call
    - if there is no next call, step to ret

## note to self
this feels so fucking great to step this way, i think just showing this shows
how great debugging using this could be

# Setup
Most is not documented, if you find anything that's not obvious, please just
send me an email or open an issue!

## Build the docker container(s)
We use an augmented qemu to trace binaries, so compile them:
```
cd tools
docker build --platform linux/arm64 . -t rebg:arm64
docker build --platform linux/amd64 . -t rebg:amd64
```

# Developing QEMU in docker
To spawn a container you can build and run qemu inside, you can use the following commands

## Start container
```sh
cd tools
docker compose up --build -d
```

## Exec into container
```sh
docker compose exec develop /bin/bash
```

## Configure & compile
```sh
root@54541497458c:~/qemu# ./configure --with-git-submodules=ignore --enable-tcg-interpreter --target-list=aarch64-linux-user,x86_64-linux-user
root@54541497458c:~/qemu# make -j $(nproc)
```

## Run & test
```sh
root@54541497458c:~/qemu# ./build/aarch64-linux-user/qemu-aarch64 /bin/ls
```

## Test with nc
First, spawn a listener. For some reason ipv4 requests to v6 listener is
automatically translated to ipv6.
```sh
root@94772de42933:~/qemu# while true; do echo -e '\n\n===='; nc -6 -lnvp 1337; done
====
Listening on :: 1337
Connection received on ::ffff:127.0.0.1 48382
elflibload|/usr/bin/ls|5500000000|550202f53f
elflibload|/lib/ld-linux-aarch64.so.1|5502831000|550286e36f
regs|pc=5502848c40|r0=0|r1=0|r2=0|r3=0|r4=0|r5=0|r6=0|r7=0|r8=0|r9=0|r10=0|r11=0|r12=0|r13=0|r14=0|r15=0|r16=0|r17=0|r18=0|r19=0|r20=0|r21=0|r22=0|r23=0|r24=0|r25=0|r26=0|r27=0|r28=0|r29=0|r30=0|r31=5502830740|flags=40000010
```

Then, somewhere else, build and run!
```sh
root@94772de42933:~/qemu# make -j $(nproc) && ./build/aarch64-linux-user/qemu-aarch64 -rebgtcp localhost:1337 -rebglog /dev/null /bin/ls
```

This mounts the folder in qemu so you can edit in your normal editor and build it and test it in the docker.

# Developer debugging tips
Use a computer or VPS with a lot of cores to debug qemu, it's going to compile
a lot faster. At least spawn a persistent docker if you you're not natively on
linux, so you don't have to recompile *everything* for every little change.

QEMU has a lot of weird indirection, and the patches I've made are quite hacky.
You can debug it in GDB though (again, use a VPS if you don't have x86_64 at
home, ptrace doesn't work in qemu-system, which docker uses). For instance
debugging what printed something to stderr:

```c
(gdb) b write if $rdi==2
Breakpoint 2 at 0x7ffff7c4ba20: file ../sysdeps/unix/sysv/linux/write.c, line 25.
(gdb) r
Thread 1 "qemu-x86_64" hit Breakpoint 2, __GI___libc_write (fd=2, buf=0x7fffffffb5f0, nbytes=1) at ../sysdeps/unix/sysv/linux/write.c:25
(gdb) bt
#0  __GI___libc_write (fd=2, buf=0x7fffffffb5f0, nbytes=1) at ../sysdeps/unix/sysv/linux/write.c:25
#8  0x000055555569e10d in vfprintf (__ap=0x7fffffffd630, __fmt=0x5555556d9e6a "{", __stream=0x7ffff7d516a0 <_IO_2_1_stderr_>) at /usr/include/x86_64-linux-gnu/bits/stdio2.h:135
#9  qemu_log (fmt=fmt@entry=0x5555556d9e6a "{") at ../util/log.c:153
#10 0x0000555555665b85 in thunk_print (arg=0x4002823fe0, type_ptr=0x55555578416c <ioctl_entries+268>, type_ptr@entry=0x555555784164 <ioctl_entries+260>)
    at ../linux-user/thunk.c:417
#11 0x0000555555650368 in print_syscall_ret_ioctl (cpu_env=<optimized out>, name=<optimized out>, ret=0, arg0=<optimized out>, arg1=<optimized out>, arg2=274919997408, 
    arg3=274922302144, arg4=180, arg5=0) at ../linux-user/strace.c:959
#12 0x000055555565135a in print_syscall_ret (cpu_env=cpu_env@entry=0x555555836660, num=num@entry=16, ret=ret@entry=0, arg1=arg1@entry=1, arg2=arg2@entry=21523, 
    arg3=arg3@entry=274919997408, arg4=274922302144, arg5=180, arg6=0) at ../linux-user/strace.c:4153
#13 0x00005555556650aa in do_syscall (cpu_env=cpu_env@entry=0x555555836660, num=16, arg1=1, arg2=21523, arg3=274919997408, arg4=274922302144, arg5=180, arg6=0, arg7=0, arg8=0)
    at ../linux-user/syscall.c:13379
#14 0x000055555558f844 in cpu_loop (env=env@entry=0x555555836660) at ../linux-user/x86_64/../i386/cpu_loop.c:233
#15 0x000055555558b78d in main (argc=<optimized out>, argv=<optimized out>, envp=<optimized out>) at ../linux-user/main.c:968
```

Edited for brievety, but this lets you trace down stuff real quick.