#include "calc_keys.inc"

.equ ST_ACC, 0
.equ ST_ENTRY, 8
.equ ST_RHS, 16
.equ ST_ACC_SIGN, 24
.equ ST_ENTRY_SIGN, 25
.equ ST_RHS_SIGN, 26
.equ ST_DOT, 27
.equ ST_INT_DIGITS, 28
.equ ST_FRAC_DIGITS, 29
.equ ST_OP, 30
.equ ST_LAST_OP, 31
.equ ST_TYPING, 32
.equ ST_ERROR, 33
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
    cbz w0, fmt_acc
    ldr x0, [x19, #ST_ENTRY]
    ldrb w1, [x19, #ST_FRAC_DIGITS]
    bl scale_entry
    mov x21, x0
    ldrb w0, [x19, #ST_ENTRY_SIGN]
    b fmt_have_value
fmt_acc:
    ldr x21, [x19, #ST_ACC]
    ldrb w0, [x19, #ST_ACC_SIGN]
fmt_have_value:
    cbz w0, fmt_no_sign
    cbz x21, fmt_no_sign
    mov w0, #45
    strb w0, [x20], #1
fmt_no_sign:
    mov x22, #SCALE
    udiv x23, x21, x22
    mul x24, x23, x22
    sub x24, x21, x24
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
    mul x6, x0, x25
    sub x24, x24, x6
    add w0, w0, #48
    strb w0, [x20], #1
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

.global scale_entry
.type scale_entry, %function
scale_entry:
    mov x2, #FRAC_DIGITS_MAX
    sub w2, w2, w1
    cmp w2, #FRAC_DIGITS_MAX
    b.hi scale_done
    mov x3, #10
scale_loop:
    cbz w2, scale_done
    mul x0, x0, x3
    subs w2, w2, #1
    b.ne scale_loop
scale_done:
    ret

.global udiv128
.type udiv128, %function
udiv128:
    cbz x0, udiv128_narrow
    mov x3, #0
    mov x4, #0
    mov x5, #128
udiv128_loop:
    lsr x6, x0, #63
    lsl x4, x4, #1
    orr x4, x4, x6
    lsl x1, x1, #1
    lsl x0, x0, #1
    cmp x4, x2
    b.lo udiv128_below
    sub x4, x4, x2
    mov x6, #1
    b udiv128_next
udiv128_below:
    mov x6, #0
udiv128_next:
    lsl x3, x3, #1
    orr x3, x3, x6
    subs x5, x5, #1
    b.ne udiv128_loop
    mov x0, x3
    mov x1, x4
    ret
udiv128_narrow:
    udiv x3, x1, x2
    msub x4, x3, x2, x1
    mov x0, x3
    mov x1, x4
    ret

.global apply_op
.type apply_op, %function
apply_op:
    stp x29, x30, [sp, #-64]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    stp x23, x24, [sp, #48]
    mov w19, w0
    mov x20, x1
    mov w21, w2
    mov x22, x3
    mov w23, w4
    cmp w19, #OP_ADD
    b.eq ap_add
    cmp w19, #OP_SUB
    b.eq ap_sub
    cmp w19, #OP_MUL
    b.eq ap_mul
    b ap_div
ap_add:
    cmp w21, w23
    b.eq ap_add_same
    cmp x20, x22
    b.hs ap_add_ge
    sub x0, x22, x20
    mov w1, w23
    b ap_ret
ap_add_ge:
    sub x0, x20, x22
    mov w1, w21
    b ap_ret
ap_add_same:
    adds x0, x20, x22
    b.cs ap_err
    mov w1, w21
    b ap_ret
ap_sub:
    eor w23, w23, #1
    b ap_add
ap_mul:
    mov x24, #SCALE
    umulh x0, x20, x22
    mul x1, x20, x22
    cmp x0, x24
    b.hs ap_err
    mov x2, x24
    bl udiv128
    mov x24, x0
    eor w1, w21, w23
    mov x0, x24
    mov w2, #0
    b ap_done
ap_div:
    cbz x22, ap_err
    mov x24, #SCALE
    umulh x0, x20, x24
    mul x1, x20, x24
    cmp x0, x22
    b.hs ap_err
    cbz x0, ap_div_narrow
    mov x2, x22
    bl udiv128
    mov x24, x0
    eor w1, w21, w23
    mov x0, x24
    mov w2, #0
    b ap_done
ap_div_narrow:
    udiv x24, x1, x22
    msub x0, x24, x22, x1
    lsl x0, x0, #1
    cmp x0, x22
    b.lo ap_div_rounded
    add x24, x24, #1
ap_div_rounded:
    eor w1, w21, w23
    mov x0, x24
    mov w2, #0
    b ap_done
ap_err:
    mov x0, #0
    mov w1, #0
    mov w2, #1
    b ap_done
ap_ret:
    mov w2, #0
ap_done:
    ldp x23, x24, [sp, #48]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #64
    ret

.global calc_key
.type calc_key, %function
calc_key:
    cmp w1, #KEY_CLEAR
    b.eq key_clear
    ldrb w2, [x0, #ST_ERROR]
    cbz w2, ck_dispatch
    ret
ck_dispatch:
    cmp w1, #KEY_9
    b.hi ck_special
    b key_digit
ck_special:
    cmp w1, #KEY_DOT
    b.eq key_dot
    cmp w1, #KEY_EQUALS
    b.eq key_equals
    cmp w1, #KEY_ADD
    b.eq key_add
    cmp w1, #KEY_SUB
    b.eq key_sub
    cmp w1, #KEY_MUL
    b.eq key_mul
    cmp w1, #KEY_DIV
    b.eq key_div
    cmp w1, #KEY_SIGN
    b.eq key_sign
    ret
key_add:
    mov w1, #OP_ADD
    b key_operator
key_sub:
    mov w1, #OP_SUB
    b key_operator
key_mul:
    mov w1, #OP_MUL
    b key_operator
key_div:
    mov w1, #OP_DIV
    b key_operator

key_digit:
    mov w2, w1
    ldrb w3, [x0, #ST_TYPING]
    cbnz w3, kd_typing
    str x2, [x0, #ST_ENTRY]
    mov w3, #1
    strb w3, [x0, #ST_TYPING]
    strb wzr, [x0, #ST_DOT]
    strb w3, [x0, #ST_INT_DIGITS]
    strb wzr, [x0, #ST_FRAC_DIGITS]
    strb wzr, [x0, #ST_ENTRY_SIGN]
    ret
kd_typing:
    ldrb w3, [x0, #ST_DOT]
    cbz w3, kd_int
    ldrb w3, [x0, #ST_FRAC_DIGITS]
    cmp w3, #FRAC_DIGITS_MAX
    b.lo kd_frac_append
    ldr x5, [x0, #ST_ENTRY]
    mov x4, #10
    madd x5, x5, x4, x2
    add x5, x5, #5
    udiv x5, x5, x4
    str x5, [x0, #ST_ENTRY]
    ret
kd_frac_append:
    ldr x5, [x0, #ST_ENTRY]
    mov x4, #10
    madd x5, x5, x4, x2
    str x5, [x0, #ST_ENTRY]
    add w3, w3, #1
    strb w3, [x0, #ST_FRAC_DIGITS]
    ret
kd_int:
    ldrb w3, [x0, #ST_INT_DIGITS]
    cmp w3, #INT_DIGITS_MAX
    b.hs kd_done
    ldr x5, [x0, #ST_ENTRY]
    mov x4, #10
    madd x5, x5, x4, x2
    str x5, [x0, #ST_ENTRY]
    add w3, w3, #1
    strb w3, [x0, #ST_INT_DIGITS]
kd_done:
    ret

key_dot:
    ldrb w2, [x0, #ST_TYPING]
    cbnz w2, kdot_typing
    str xzr, [x0, #ST_ENTRY]
    mov w2, #1
    strb w2, [x0, #ST_TYPING]
    strb w2, [x0, #ST_DOT]
    strb w2, [x0, #ST_INT_DIGITS]
    strb wzr, [x0, #ST_FRAC_DIGITS]
    strb wzr, [x0, #ST_ENTRY_SIGN]
    ret
kdot_typing:
    ldrb w2, [x0, #ST_DOT]
    cbz w2, kdot_set
    ret
kdot_set:
    mov w2, #1
    strb w2, [x0, #ST_DOT]
    ret

key_sign:
    ldrb w2, [x0, #ST_TYPING]
    cbz w2, ks_acc
    ldrb w2, [x0, #ST_ENTRY_SIGN]
    eor w2, w2, #1
    strb w2, [x0, #ST_ENTRY_SIGN]
    ret
ks_acc:
    ldrb w2, [x0, #ST_ACC_SIGN]
    eor w2, w2, #1
    strb w2, [x0, #ST_ACC_SIGN]
    ret

key_clear:
    b calc_reset

.global key_operator
.type key_operator, %function
key_operator:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    mov x19, x0
    mov w20, w1
    ldrb w21, [x19, #ST_TYPING]
    cbz w21, ko_set
    ldrb w22, [x19, #ST_OP]
    cbz w22, ko_first
    ldr x0, [x19, #ST_ENTRY]
    ldrb w1, [x19, #ST_FRAC_DIGITS]
    bl scale_entry
    mov x3, x0
    ldr x1, [x19, #ST_ACC]
    ldrb w2, [x19, #ST_ACC_SIGN]
    ldrb w4, [x19, #ST_ENTRY_SIGN]
    mov w0, w22
    bl apply_op
    cbz w2, ko_store
    mov w3, #1
    strb w3, [x19, #ST_ERROR]
    b ko_done
ko_store:
    str x0, [x19, #ST_ACC]
    strb w1, [x19, #ST_ACC_SIGN]
    b ko_clear
ko_first:
    ldr x0, [x19, #ST_ENTRY]
    ldrb w1, [x19, #ST_FRAC_DIGITS]
    bl scale_entry
    str x0, [x19, #ST_ACC]
    ldrb w1, [x19, #ST_ENTRY_SIGN]
    strb w1, [x19, #ST_ACC_SIGN]
ko_clear:
    strb wzr, [x19, #ST_TYPING]
    strb wzr, [x19, #ST_DOT]
    strb wzr, [x19, #ST_ENTRY_SIGN]
    strb wzr, [x19, #ST_FRAC_DIGITS]
    mov w3, #1
    strb w3, [x19, #ST_INT_DIGITS]
    str xzr, [x19, #ST_ENTRY]
ko_set:
    strb w20, [x19, #ST_OP]
ko_done:
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global key_equals
.type key_equals, %function
key_equals:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    mov x19, x0
    ldrb w21, [x19, #ST_TYPING]
    cbz w21, ke_repeat
    ldr x0, [x19, #ST_ENTRY]
    ldrb w1, [x19, #ST_FRAC_DIGITS]
    bl scale_entry
    str x0, [x19, #ST_RHS]
    ldrb w1, [x19, #ST_ENTRY_SIGN]
    strb w1, [x19, #ST_RHS_SIGN]
    ldrb w22, [x19, #ST_OP]
    cbz w22, ke_first
    mov w0, w22
    ldr x3, [x19, #ST_RHS]
    ldrb w4, [x19, #ST_RHS_SIGN]
    ldr x1, [x19, #ST_ACC]
    ldrb w2, [x19, #ST_ACC_SIGN]
    bl apply_op
    cbz w2, ke_store
    mov w3, #1
    strb w3, [x19, #ST_ERROR]
    b ke_done
ke_store:
    str x0, [x19, #ST_ACC]
    strb w1, [x19, #ST_ACC_SIGN]
    strb w22, [x19, #ST_LAST_OP]
    b ke_finish
ke_first:
    ldr x0, [x19, #ST_ENTRY]
    ldrb w1, [x19, #ST_FRAC_DIGITS]
    bl scale_entry
    str x0, [x19, #ST_ACC]
    ldrb w1, [x19, #ST_ENTRY_SIGN]
    strb w1, [x19, #ST_ACC_SIGN]
ke_finish:
    strb wzr, [x19, #ST_TYPING]
    strb wzr, [x19, #ST_DOT]
    strb wzr, [x19, #ST_ENTRY_SIGN]
    strb wzr, [x19, #ST_FRAC_DIGITS]
    mov w3, #1
    strb w3, [x19, #ST_INT_DIGITS]
    str xzr, [x19, #ST_ENTRY]
    strb wzr, [x19, #ST_OP]
    b ke_done
ke_repeat:
    ldrb w22, [x19, #ST_LAST_OP]
    cbz w22, ke_done
    mov w0, w22
    ldr x3, [x19, #ST_RHS]
    ldrb w4, [x19, #ST_RHS_SIGN]
    ldr x1, [x19, #ST_ACC]
    ldrb w2, [x19, #ST_ACC_SIGN]
    bl apply_op
    cbz w2, ke_store_repeat
    mov w3, #1
    strb w3, [x19, #ST_ERROR]
    b ke_done
ke_store_repeat:
    str x0, [x19, #ST_ACC]
    strb w1, [x19, #ST_ACC_SIGN]
ke_done:
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.section .rodata
str_error:
    .asciz "Error"
