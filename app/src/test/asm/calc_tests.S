.equ SYS_WRITE, 64
.equ SYS_EXIT, 93
.equ STDOUT, 1

.equ NUMBUF_SIZE, 32

.section .bss
.balign 8
total_cases:
    .quad 0
failed_cases:
    .quad 0
numbuf:
    .space NUMBUF_SIZE
test_state:
    .space 40
test_out:
    .space 64

.section .rodata
sf_a:
    .asciz "abc"
sf_b:
    .asciz "abd"
sf_empty:
    .asciz ""
sf_prefix:
    .asciz "PASS "
sf_fail_prefix:
    .asciz "FAIL "
sf_expected:
    .asciz ": expected \""
sf_got:
    .asciz "\" got \""
sf_quote:
    .asciz "\"\n"
sf_newline:
    .asciz "\n"
sf_harness_ok:
    .asciz "harness: comparison and reporting primitives"
sf_harness_bad:
    .asciz "harness: comparison primitives are broken"
sf_summary:
    .asciz "cases: "
sf_failed:
    .asciz " failed: "
sf_all_passed:
    .asciz "all tests passed\n"

.section .text
.global print
.type print, %function
print:
    mov x2, x1
    mov x1, x0
    mov x0, #STDOUT
    mov x8, #SYS_WRITE
    svc #0
    ret

.global print_cstr
.type print_cstr, %function
print_cstr:
    mov x1, x0
pc_len:
    ldrb w2, [x0]
    cbz w2, pc_len_done
    add x0, x0, #1
    b pc_len
pc_len_done:
    mov x0, x1
    b print

.global cstr_len
.type cstr_len, %function
cstr_len:
    mov x1, xzr
cl_next:
    ldrb w2, [x0, x1]
    cbz w2, cl_done
    add x1, x1, #1
    b cl_next
cl_done:
    mov x0, x1
    ret

.global print_u64
.type print_u64, %function
print_u64:
    adrp x1, numbuf
    add x1, x1, :lo12:numbuf
    add x2, x1, #NUMBUF_SIZE
    mov x3, #10
    mov x4, x2
pu_loop:
    udiv x5, x0, x3
    msub x6, x5, x3, x0
    add x6, x6, #48
    sub x4, x4, #1
    strb w6, [x4]
    mov x0, x5
    cbnz x0, pu_loop
    sub x1, x2, x4
    mov x0, x4
    b print

.global cstr_eq
.type cstr_eq, %function
cstr_eq:
    mov x2, x0
    mov x3, x1
    mov x0, xzr
ce_len_a:
    ldrb w4, [x2, x0]
    cbz w4, ce_len_done
    add x0, x0, #1
    b ce_len_a
ce_len_done:
    mov x4, x0
    mov x0, xzr
ce_cmp:
    cmp x0, x4
    b.ge ce_tail_equal
    ldrb w5, [x2, x0]
    ldrb w6, [x3, x0]
    cmp w5, w6
    b.ne ce_false
    add x0, x0, #1
    b ce_cmp
ce_tail_equal:
    ldrb w5, [x3, x0]
    cbnz w5, ce_false
ce_true:
    mov x0, #1
    ret
ce_false:
    mov x0, #0
    ret

.global report_pass
.type report_pass, %function
report_pass:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    mov x19, x0
    adrp x0, total_cases
    add x0, x0, :lo12:total_cases
    ldr x1, [x0]
    add x1, x1, #1
    str x1, [x0]
    adrp x0, sf_prefix
    add x0, x0, :lo12:sf_prefix
    bl print_cstr
    mov x0, x19
    bl print_cstr
    adrp x0, sf_newline
    add x0, x0, :lo12:sf_newline
    bl print_cstr
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global report_fail
.type report_fail, %function
report_fail:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    mov x19, x0
    mov x20, x1
    mov x21, x2
    adrp x0, total_cases
    add x0, x0, :lo12:total_cases
    ldr x1, [x0]
    add x1, x1, #1
    str x1, [x0]
    adrp x0, failed_cases
    add x0, x0, :lo12:failed_cases
    ldr x1, [x0]
    add x1, x1, #1
    str x1, [x0]
    adrp x0, sf_fail_prefix
    add x0, x0, :lo12:sf_fail_prefix
    bl print_cstr
    mov x0, x19
    bl print_cstr
    adrp x0, sf_expected
    add x0, x0, :lo12:sf_expected
    bl print_cstr
    mov x0, x21
    bl print_cstr
    adrp x0, sf_got
    add x0, x0, :lo12:sf_got
    bl print_cstr
    mov x0, x20
    bl print_cstr
    adrp x0, sf_quote
    add x0, x0, :lo12:sf_quote
    bl print_cstr
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global check
.type check, %function
check:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    mov x19, x0
    mov x20, x1
    mov x21, x2
    mov x0, x20
    mov x1, x21
    bl cstr_eq
    cbz x0, check_failed
    mov x0, x19
    bl report_pass
    mov x0, #1
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret
check_failed:
    mov x0, x19
    mov x1, x20
    mov x2, x21
    bl report_fail
    mov x0, #0
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global print_summary
.type print_summary, %function
print_summary:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    adrp x0, sf_summary
    add x0, x0, :lo12:sf_summary
    bl print_cstr
    adrp x0, total_cases
    add x0, x0, :lo12:total_cases
    ldr x0, [x0]
    bl print_u64
    adrp x0, sf_failed
    add x0, x0, :lo12:sf_failed
    bl print_cstr
    adrp x0, failed_cases
    add x0, x0, :lo12:failed_cases
    ldr x0, [x0]
    bl print_u64
    adrp x0, sf_newline
    add x0, x0, :lo12:sf_newline
    bl print_cstr
    adrp x0, failed_cases
    add x0, x0, :lo12:failed_cases
    ldr x19, [x0]
    cbz x19, summary_ok
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret
summary_ok:
    adrp x0, sf_all_passed
    add x0, x0, :lo12:sf_all_passed
    bl print_cstr
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global framework_selftest
.type framework_selftest, %function
framework_selftest:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    adrp x0, sf_a
    add x0, x0, :lo12:sf_a
    adrp x1, sf_a
    add x1, x1, :lo12:sf_a
    bl cstr_eq
    cbz x0, fs_fail
    adrp x0, sf_a
    add x0, x0, :lo12:sf_a
    adrp x1, sf_b
    add x1, x1, :lo12:sf_b
    bl cstr_eq
    cbnz x0, fs_fail
    adrp x0, sf_a
    add x0, x0, :lo12:sf_a
    adrp x1, sf_empty
    add x1, x1, :lo12:sf_empty
    bl cstr_eq
    cbnz x0, fs_fail
    adrp x0, sf_empty
    add x0, x0, :lo12:sf_empty
    adrp x1, sf_empty
    add x1, x1, :lo12:sf_empty
    bl cstr_eq
    cbz x0, fs_fail
    adrp x0, sf_a
    add x0, x0, :lo12:sf_a
    bl cstr_len
    cmp x0, #3
    b.ne fs_fail
    adrp x0, sf_empty
    add x0, x0, :lo12:sf_empty
    bl cstr_len
    cbnz x0, fs_fail
    adrp x0, sf_harness_ok
    add x0, x0, :lo12:sf_harness_ok
    bl report_pass
    ldp x29, x30, [sp], #48
    ret
fs_fail:
    adrp x0, sf_harness_bad
    add x0, x0, :lo12:sf_harness_bad
    adrp x1, sf_a
    add x1, x1, :lo12:sf_a
    adrp x2, sf_b
    add x2, x2, :lo12:sf_b
    bl report_fail
    ldp x29, x30, [sp], #48
    ret

.global run_case
.type run_case, %function
run_case:
    stp x29, x30, [sp, #-64]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    stp x23, x24, [sp, #48]
    mov x19, x0
    mov x20, x1
    mov x21, x2
    mov x22, x3
    adrp x0, test_state
    add x0, x0, :lo12:test_state
    bl calc_reset
    mov x0, xzr
rc_key_loop:
    cmp x0, x21
    b.ge rc_keys_done
    ldrb w1, [x20, x0]
    adrp x2, test_state
    add x2, x2, :lo12:test_state
    mov x0, x2
    bl calc_key
    add x0, x0, #1
    b rc_key_loop
rc_keys_done:
    adrp x0, test_state
    add x0, x0, :lo12:test_state
    adrp x1, test_out
    add x1, x1, :lo12:test_out
    bl calc_format
    mov x0, x19
    adrp x1, test_out
    add x1, x1, :lo12:test_out
    mov x2, x22
    bl check
    ldp x23, x24, [sp, #48]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #64
    ret

.global calc_test_main
.type calc_test_main, %function
calc_test_main:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    bl framework_selftest
    adrp x0, test_table
    add x0, x0, :lo12:test_table
    ldr x1, [x0, #16]
    cbz x1, table_done
table_next:
    ldr x19, [x0]
    ldr x20, [x0, #8]
    ldr x21, [x0, #16]
    ldr x22, [x0, #24]
    bl run_case
    add x0, x0, #32
    ldr x1, [x0, #16]
    cbnz x1, table_next
table_done:
    bl print_summary
    adrp x0, failed_cases
    add x0, x0, :lo12:failed_cases
    ldr x0, [x0]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.section .rodata
.balign 8
keys_reset:
    .byte 16
keys_clear_after_digits:
    .byte 1, 2, 3, 16
.balign 8
t_reset:
    .asciz "reset shows zero"
t_clear:
    .asciz "clear returns to zero"
t_clear_after_digits:
    .asciz "clear after digits returns to zero"
.balign 8
test_table:
    .quad t_reset
    .quad keys_reset
    .quad 1
    .quad s_zero
    .quad t_clear
    .quad keys_reset
    .quad 1
    .quad s_zero
    .quad t_clear_after_digits
    .quad keys_clear_after_digits
    .quad 4
    .quad s_zero
    .quad 0
    .quad 0
    .quad 0
    .quad 0

.section .rodata
s_zero:
    .asciz "0"
