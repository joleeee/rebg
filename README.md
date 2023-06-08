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