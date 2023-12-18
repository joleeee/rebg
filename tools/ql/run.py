#!/usr/bin/env python3

from qiling import Qiling
from qiling.const import QL_ARCH, QL_INTERCEPT, QL_VERBOSE
from qiling.extensions import pipe
from qiling.os.mapper import QlFsMappedObject
from typing import List
from pprint import pprint
import os

# from capstone import *
from unicorn import x86_const, arm64_const

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

from enum import Enum


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


ser = Serializer()


def code(ql: Qiling, address, size):
    ser.separator()

    ser.address(address)

    buf = ql.uc.mem_read(address, size)
    ser.code(buf)

    regs = []
    for r in arch.regs():
        regs.append(ql.arch.regs.read(r))

    pc = ql.arch.regs.arch_pc

    flags = ql.arch.regs.read(arch.flags_reg())

    ser.registers(flags, pc, regs)


def mem_read(ql, access, adr, size, value):
    assert size in [0x1, 0x2, 0x4, 0x8]
    # print(f"READ at {adr:#x} {size=:#x}, {value=:#x}")
    ser.load(adr, value, size)


def mem_write(ql, access, adr, size, value):
    assert size in [0x1, 0x2, 0x4, 0x8]
    # print(f"WRITE at {adr:#x} {size=:#x}, {value=:#x}")
    ser.store(adr, value, size)


def inter(ql, a):
    # print("INT", a)
    pass


arch = None

def sys_openat(ql, fd, path, flags, mode, ret):
    path = ql.mem.string(path)
    syscall = f'openat(0x{fd:x}, "{path}", 0x{flags:x}, 0x{mode:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_new_fstatat(ql, fd, path, buf, flags, ret):
    path = ql.mem.string(path)
    syscall = f'new_fstatat(0x{fd:x}, "{path}", 0x{buf:x}, 0x{flags:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_read(ql, fd, buf, count, ret):
    syscall = f'read(0x{fd:x}, 0x{buf:x}, 0x{count:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_write(ql, fd, buf, count, ret):
    syscall = f'write(0x{fd:x}, 0x{buf:x}, 0x{count:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_brk(ql, address, ret):
    syscall = f'brk(0x{address:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_exit_group(ql, status, ret):
    syscall = f'exit_group({status}) = {ret}'
    ser.syscall(syscall.encode())

def sys_close(ql, fd, ret):
    syscall = f'close({fd}) = {ret}'
    ser.syscall(syscall.encode())

def sys_mmap(ql, addr, length, prot, flags, fd, offset, ret):
    syscall = f'mmap(0x{addr:x}, 0x{length:x}, 0x{prot:x}, 0x{flags:x}, {fd}, 0x{offset:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_uname(ql, buf, ret):
    syscall = f'uname(0x{buf:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())

def sys_mprotect(ql, addr, len, prot, ret):
    syscall = f'mprotect(0x{addr:x}, 0x{len:x}, 0x{prot:x}) = 0x{ret:x}'
    ser.syscall(syscall.encode())


ENABLE = False
def enable_rebg(ql):
    global ENABLE
    if ENABLE:
        return
    ENABLE = True

    ql.hook_code(code, begin=bin_low, end=bin_high)
    ql.hook_mem_read(mem_read)
    ql.hook_mem_write(mem_write)

    # TODO find a nice way to do all of the syscalls
    for name, func in [
        ("brk", sys_brk),
        ("exit_group", sys_exit_group),
        ("close", sys_close),
        ("mmap", sys_mmap),
        ("uname", sys_uname),
        ("mprotect", sys_mprotect),
        ("openat", sys_openat),
        ("read", sys_read),
        ("write", sys_write),
    ]:
        ql.os.set_syscall(name, func, QL_INTERCEPT.EXIT)


def try_patch_isa(ql: Qiling):
    # 00023881  8b8a28030000       mov     ecx, dword [rdx+0x328]
    # 00023887  89cf               mov     edi, ecx
    # 00023889  4421c7             and     edi, r8d  {0x0}
    # 0002388c  39f9               cmp     ecx, edi
    # 0002388e  0f85000b0000       jne     0x24394  // from here

    # 00023894  488d50f8           lea     rdx, [rax-0x8]
    # 00023898  4839f0             cmp     rax, rsi
    # 0002389b  75c3               jne     0x23860

    # 0002389d  4c89ce             mov     rsi, r9  {0x3010102464c457f}  // to here
    # 000238a0  4c89ff             mov     rdi, r15
    # 000238a3  e8d872ffff         call    sub_1ab80

    pre = bytes.fromhex("8b8a2803000089cf4421c739f9")
    ins = bytes.fromhex("0f85000b0000")
    skip = bytes.fromhex("488d50f84839f075c3")

    def bypass_isa_check(ql: Qiling) -> None:
        print("Bypassing ISA Check...")
        ql.arch.regs.rip += len(ins) + len(skip)

    for start, end, perm, label, img in ql.mem.get_mapinfo():
        if label != "ld-linux-x86-64.so.2":
            continue
        if "x" not in perm:
            continue

        adrs = ql.mem.search(pre + ins + skip, begin=start, end=end)
        for adr in adrs:
            ql.hook_address(bypass_isa_check, adr + len(pre))


def run(rootfs, argv):
    ql = Qiling(argv, rootfs)

    global arch
    arch = Arch(ql.arch.type)

    realpath_bin = os.path.realpath(argv[0])

    binary_offsets = [
        (start, end)
        for start, end, _, _, img in ql.mem.get_mapinfo()
        if realpath_bin == img
    ]

    # simplification of reality
    global bin_low, bin_high
    bin_low = min([start for start, _ in binary_offsets])
    bin_high = max([end for _, end in binary_offsets])

    for start, end, perm, label, img in ql.mem.get_mapinfo():
        if len(img) == 0:
            continue

        file = img.encode()
        ser.libload(file, start, end)

    # ql.hook_address(lambda ql: enable_rebg(ql), 0x00007FFFB7EEA5F0)
    # ql.os.stdin = pipe.SimpleInStream(0)
    # ql.os.stdin.write(b"abcde\n")

    try_patch_isa(ql)
    enable_rebg(ql)
    ql.run()


if __name__ == "__main__":
    from sys import argv

    if len(argv) < 3:
        print("not enough args")
        exit(0)

    rootfs = argv[1]
    rest = argv[2:]

    run(rootfs, rest)
