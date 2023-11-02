.section .data
start_msg:
    .asciz "which char am i thinking about?\n"
win_msg:
    .asciz "you win!!\n"
lose_msg:
    .asciz "try again\n"

.section .bss
buf:
    .skip 1

.section .text
.global _start

_start:
    mov x8, 64
    mov x0, 1
    ldr x1, =start_msg
    mov x2, 32 // len
    svc 0
    
    // read 1 char
    mov x8, 63
    mov x0, 0
    ldr x1, =buf
    mov x2, 1 // len
    svc 0

    // check if the char is 'r'
    ldrb w20, [x1]
    cmp x20, #'r'
    b.eq win
    b lose

win:
    mov x8, 64
    mov x0, 1
    ldr x1, =win_msg
    mov x2, 10 // len
    svc 0
    b exit

lose:
    mov x8, 64
    mov x0, 1
    ldr x1, =lose_msg
    mov x2, 10 // len
    svc 0
    b exit

exit:
    mov x8, 93
    mov x0, 0
    svc 0
