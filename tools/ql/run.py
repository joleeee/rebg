#!/usr/bin/env python3

from qiling import Qiling
from qiling.extensions import pipe
from qiling.os.mapper import QlFsMappedObject
import rebg


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
    rb = rebg.Rebg(ql)

    # ql.hook_address(lambda ql: enable_rebg(ql), 0x00007FFFB7EEA5F0)
    # ql.os.stdin = pipe.SimpleInStream(0)
    # ql.os.stdin.write(b"abcde\n")

    try_patch_isa(ql)
    rb.enable()
    ql.run()


if __name__ == "__main__":
    from sys import argv

    if len(argv) < 3:
        print("not enough args")
        exit(0)

    rootfs = argv[1]
    rest = argv[2:]

    run(rootfs, rest)
