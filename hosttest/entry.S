.section .text.start
.global _start
.type _start, %function
_start:
    ldr x0, [sp]
    add x1, sp, #8
    mov w0, #'A'
    bl mark
    bl calc_test_main
    mov w0, #'E'
    bl mark
    mov x8, #93
    svc #0
