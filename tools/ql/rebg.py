#!/usr/bin/env python3

from qiling import Qiling
from qiling.const import QL_ARCH, QL_INTERCEPT, QL_VERBOSE
from typing import List
from unicorn import x86_const, arm64_const
from enum import Enum
import os

X86_REGS = [
    "rax",
    "rcx",
    "rdx",
    "rbx",
    "rsp",
    "rbp",
    "rsi",
    "rdi",
    "r8",
    "r9",
    "r10",
    "r11",
    "r12",
    "r13",
    "r14",
    "r15",
]

ARM64_REGS = [
    "x0",
    "x1",
    "x2",
    "x3",
    "x4",
    "x5",
    "x6",
    "x7",
    "x8",
    "x9",
    "x10",
    "x11",
    "x12",
    "x13",
    "x14",
    "x15",
    "x16",
    "x17",
    "x18",
    "x19",
    "x20",
    "x21",
    "x22",
    "x23",
    "x24",
    "x25",
    "x26",
    "x27",
    "x28",
    "x29",  # fp
    "lr",
    "sp",
]


class Arch(Enum):
    ARM64 = QL_ARCH.ARM64
    X8664 = QL_ARCH.X8664

    def flags_reg(self):
        if self == self.ARM64:
            return arm64_const.UC_ARM64_REG_NZCV
        elif self == self.X8664:
            return x86_const.UC_X86_REG_FLAGS
        else:
            raise Exception("what u doin")

    def regs(self):
        if self == self.ARM64:
            return ARM64_REGS
        elif self == self.X8664:
            return X86_REGS
        else:
            raise Exception("what u doin")


class Serializer:
    def __init__(self):
        import socket

        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(("localhost", 1337))

        self.sock = sock

    def separator(self):
        self.sock.sendall(b"\x55")

    def address(self, value: int):
        self.sock.sendall(b"\xaa")
        self.sock.sendall(value.to_bytes(8, "little"))

    def code(self, bin: bytes):
        self.sock.sendall(b"\xff")
        self.sock.sendall(len(bin).to_bytes(8, "little"))
        self.sock.sendall(bin)

    def registers(self, flags, pc, regs: List[int]):
        self.sock.sendall(b"\x77")
        self.sock.sendall(len(regs).to_bytes(1, "little"))
        self.sock.sendall(flags.to_bytes(8, "little"))
        self.sock.sendall(pc.to_bytes(8, "little"))
        for r in regs:
            self.sock.sendall(r.to_bytes(8, "little"))

    def libload(self, name: bytes, fr: int, to: int):
        self.sock.sendall(b"\xee")
        self.sock.sendall(len(name).to_bytes(8, "little"))
        self.sock.sendall(name)
        self.sock.sendall(fr.to_bytes(8, "little"))
        self.sock.sendall(to.to_bytes(8, "little"))

    def libload_bin(self, name: bytes, content: bytes, fr: int, to: int):
        self.sock.sendall(b"\xef")
        self.sock.sendall(len(name).to_bytes(8, "little"))
        self.sock.sendall(name)
        self.sock.sendall(len(content).to_bytes(8, "little"))
        self.sock.sendall(content)
        self.sock.sendall(fr.to_bytes(8, "little"))
        self.sock.sendall(to.to_bytes(8, "little"))

    def load(self, adr, value, size):
        self.sock.sendall(b"\x33")
        self.sock.sendall(size.to_bytes(1, "little"))
        self.sock.sendall(adr.to_bytes(8, "little"))
        self.sock.sendall(value.to_bytes(8, "little", signed=True))

    def store(self, adr, value, size):
        self.sock.sendall(b"\x44")
        self.sock.sendall(size.to_bytes(1, "little"))
        self.sock.sendall(adr.to_bytes(8, "little"))
        self.sock.sendall(value.to_bytes(8, "little", signed=True))

    def syscall(self, data: bytes):
        self.sock.sendall(b"\x99")
        self.sock.sendall(len(data).to_bytes(8, "little"))
        self.sock.sendall(data)


class Rebg:
    def __init__(self, ql: Qiling) -> None:
        self.enabled = False
        self.registered = False
        self.ser = Serializer()
        self.ql = ql
        self.arch = Arch(ql.arch.type)

        # register & setup
        binary_offsets = [
            (start, end)
            for start, end, _, _, img in self.ql.mem.get_mapinfo()
            if img == os.path.realpath(self.ql.argv[0])
        ]

        # simplification of reality
        self.bin_low = min([start for start, _ in binary_offsets])
        self.bin_high = max([end for _, end in binary_offsets])

        for start, end, perm, label, img in self.ql.mem.get_mapinfo():
            if len(img) == 0:
                continue

            file = img.encode()
            self.ser.libload(file, start, end)

    def enable(self):
        if self.enabled:
            return
        self.enabled = True

        self.ql.hook_code(self.code, begin=self.bin_low, end=self.bin_high)
        self.ql.hook_mem_read(self.mem_read)
        self.ql.hook_mem_write(self.mem_write)

        # TODO find a nice way to do all of the syscalls
        for name, func in [
            ("brk", self.sys_brk),
            ("exit_group", self.sys_exit_group),
            ("close", self.sys_close),
            ("mmap", self.sys_mmap),
            ("uname", self.sys_uname),
            ("mprotect", self.sys_mprotect),
            ("openat", self.sys_openat),
            ("read", self.sys_read),
            ("write", self.sys_write),
        ]:
            self.ql.os.set_syscall(name, func, QL_INTERCEPT.EXIT)

    # associated functions
    def code(self, ql: Qiling, address, size):
        self.ser.separator()

        self.ser.address(address)

        buf = ql.uc.mem_read(address, size)
        self.ser.code(buf)

        regs = []
        for r in self.arch.regs():
            regs.append(ql.arch.regs.read(r))

        pc = ql.arch.regs.arch_pc

        flags = ql.arch.regs.read(self.arch.flags_reg())

        self.ser.registers(flags, pc, regs)

    def mem_read(self, ql, access, adr, size, value):
        assert size in [0x1, 0x2, 0x4, 0x8]
        self.ser.load(adr, value, size)

    def mem_write(self, ql, access, adr, size, value):
        assert size in [0x1, 0x2, 0x4, 0x8]
        self.ser.store(adr, value, size)

    # syscalls
    def sys_openat(self, ql, fd, path, flags, mode, ret):
        path = ql.mem.string(path)
        syscall = f'openat(0x{fd:x}, "{path}", 0x{flags:x}, 0x{mode:x}) = 0x{ret:x}'
        self.ser.syscall(syscall.encode())

    def sys_new_fstatat(self, ql, fd, path, buf, flags, ret):
        path = ql.mem.string(path)
        syscall = f'new_fstatat(0x{fd:x}, "{path}", 0x{buf:x}, 0x{flags:x}) = 0x{ret:x}'
        self.ser.syscall(syscall.encode())

    def sys_read(self, ql, fd, buf, count, ret):
        syscall = f"read(0x{fd:x}, 0x{buf:x}, 0x{count:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())

    def sys_write(self, ql, fd, buf, count, ret):
        syscall = f"write(0x{fd:x}, 0x{buf:x}, 0x{count:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())

    def sys_brk(self, ql, address, ret):
        syscall = f"brk(0x{address:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())

    def sys_exit_group(self, ql, status, ret):
        syscall = f"exit_group({status}) = {ret}"
        self.ser.syscall(syscall.encode())

    def sys_close(self, ql, fd, ret):
        syscall = f"close({fd}) = {ret}"
        self.ser.syscall(syscall.encode())

    def sys_mmap(self, ql, addr, length, prot, flags, fd, offset, ret):
        syscall = f"mmap(0x{addr:x}, 0x{length:x}, 0x{prot:x}, 0x{flags:x}, {fd}, 0x{offset:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())

    def sys_uname(self, ql, buf, ret):
        syscall = f"uname(0x{buf:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())

    def sys_mprotect(self, ql, addr, len, prot, ret):
        syscall = f"mprotect(0x{addr:x}, 0x{len:x}, 0x{prot:x}) = 0x{ret:x}"
        self.ser.syscall(syscall.encode())
