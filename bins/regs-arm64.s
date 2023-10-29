.section .data
hello_message:
    .asciz "rebg rules\n"

.section .text
.global _start

_start:
    mov x8, 64
    mov x0, 1
    ldr x1, =hello_message
    mov x2, 11 // len
    svc 0

    // for testing w0 is set to write/read
    mov w0, 0x1337
    mov w1, w0

    mov x8, 93
    mov x0, 0
    svc 0
