#/bin/sh
as -o regs-arm64.o regs-arm64.s && ld -o regs-arm64 regs-arm64.o
