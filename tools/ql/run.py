#!/usr/bin/env python3

from qiling import Qiling
from qiling.const import QL_ARCH
from qiling.os.mapper import QlFsMappedObject
from typing import List
from pprint import pprint
import os

# from capstone import *
from unicorn import x86_const, arm64_const

# md = Cs(CS_ARCH_X86, CS_MODE_64)
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
    bin_low = min([start for start, _ in binary_offsets])
    bin_high = max([end for _, end in binary_offsets])

    for start, end, perm, label, img in ql.mem.get_mapinfo():
        if len(img) == 0:
            continue

        file = img.encode()
        ser.libload(file, start, end)

    # ql.run(end=0x400000)
    # snap = ql.save()
    # ql.restore(snap)

    ql.hook_code(code, begin=bin_low, end=bin_high)
    ql.hook_mem_read(mem_read)
    ql.hook_mem_write(mem_write)

    ql.os.run()


if __name__ == "__main__":
    from sys import argv

    if len(argv) < 3:
        print("not enough args")
        exit(0)

    rootfs = argv[1]
    rest = argv[2:]

    run(rootfs, rest)
