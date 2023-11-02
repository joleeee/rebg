#/bin/sh
as -o regs-arm64.o regs-arm64.s && ld -o regs-arm64 regs-arm64.o
as -o char-arm64.o char-arm64.s && ld -o char-arm64 char-arm64.o
