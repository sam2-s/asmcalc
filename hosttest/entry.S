.section .text.start
.global _start
.type _start, %function
_start:
    ldr x0, [sp]
    add x1, sp, #8
    bl calc_test_main
    mov x8, #93
    svc #0
