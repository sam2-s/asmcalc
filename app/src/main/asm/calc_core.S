#include "calc_keys.inc"

.equ ST_ACC, 0
.equ ST_ENTRY, 8
.equ ST_RHS, 16
.equ ST_OP, 24
.equ ST_LAST_OP, 25
.equ ST_TYPING, 26
.equ ST_ERROR, 27
.equ ST_SIGN, 28
.equ ST_RHS_SIGN, 29
.equ ST_DOT, 30
.equ ST_INT_DIGITS, 31
.equ ST_FRAC_DIGITS, 32
.equ ST_SIZE, 40

.equ OP_NONE, 0
.equ OP_ADD, 1
.equ OP_SUB, 2
.equ OP_MUL, 3
.equ OP_DIV, 4

.equ SCALE, 10000
.equ FRAC_DIGITS_MAX, 4
.equ INT_DIGITS_MAX, 12

.section .text
.global calc_reset
.type calc_reset, %function
calc_reset:
    mov x1, #0
    stp xzr, xzr, [x0]
    stp xzr, xzr, [x0, #16]
    stp xzr, xzr, [x0, #32]
    ret

.global u64_to_dec
.type u64_to_dec, %function
u64_to_dec:
    mov x9, x0
    mov x3, #1
    mov x4, x1
    mov x2, #10
dec_count:
    udiv x5, x4, x2
    cbz x5, dec_count_done
    mov x4, x5
    add x3, x3, #1
    b dec_count
dec_count_done:
    add x0, x9, x3
    sub x0, x0, #1
    mov x4, x1
    mov x6, #10
    mov x12, x3
dec_fill:
    udiv x7, x4, x6
    msub x5, x7, x6, x4
    add w5, w5, #48
    strb w5, [x0]
    sub x0, x0, #1
    mov x4, x7
    subs x3, x3, #1
    b.ne dec_fill
    add x0, x0, #1
    add x0, x0, x12
    sub x1, x0, x9
    ret

.global calc_format
.type calc_format, %function
calc_format:
    stp x29, x30, [sp, #-80]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    stp x23, x24, [sp, #48]
    stp x25, x26, [sp, #64]
    mov x19, x0
    mov x20, x1
    mov x26, x1
    ldrb w0, [x19, #ST_ERROR]
    cbnz w0, fmt_error
    ldrb w0, [x19, #ST_TYPING]
    cbz w0, fmt_not_typing
    ldr x21, [x19, #ST_ENTRY]
    b fmt_have_value
fmt_not_typing:
    ldr x21, [x19, #ST_ACC]
fmt_have_value:
    ldrb w0, [x19, #ST_SIGN]
    cbz w0, fmt_no_sign
    cbz x21, fmt_no_sign
    mov w0, #45
    strb w0, [x20], #1
fmt_no_sign:
    mov x22, #SCALE
    udiv x23, x21, x22
    msub x24, x23, x22, x21
    mov x0, x20
    mov x1, x23
    bl u64_to_dec
    mov x20, x0
    cbz x24, fmt_terminate
    mov w0, #46
    strb w0, [x20], #1
    mov x22, x20
    mov x25, #1000
fmt_frac_loop:
    udiv x0, x24, x25
    add w0, w0, #48
    strb w0, [x20], #1
    msub x24, x0, x25, x24
    cmp x25, #1
    b.eq fmt_frac_done
    mov x0, #10
    udiv x25, x25, x0
    b fmt_frac_loop
fmt_frac_done:
    cmp x20, x22
    b.le fmt_terminate
fmt_frac_trim:
    ldrb w0, [x20, #-1]
    cmp w0, #48
    b.ne fmt_terminate
    sub x20, x20, #1
    cmp x20, x22
    b.gt fmt_frac_trim
fmt_terminate:
    mov w0, #0
    strb w0, [x20]
    sub x0, x20, x26
    b fmt_done
fmt_error:
    mov x0, x20
    adrp x1, str_error
    add x1, x1, :lo12:str_error
    bl cstr_copy
    mov x0, #5
fmt_done:
    ldp x25, x26, [sp, #64]
    ldp x23, x24, [sp, #48]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #80
    ret

.global cstr_copy
.type cstr_copy, %function
cstr_copy:
    mov x2, xzr
cc_next:
    ldrb w3, [x1, x2]
    strb w3, [x0, x2]
    cbz w3, cc_done
    add x2, x2, #1
    b cc_next
cc_done:
    ret

.global calc_key
.type calc_key, %function
calc_key:
    cmp w1, #KEY_CLEAR
    b.eq key_clear
    ret
key_clear:
    b calc_reset

.section .rodata
str_error:
    .asciz "Error"
