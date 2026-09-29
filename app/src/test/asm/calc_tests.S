.equ SYS_WRITE, 64
.equ SYS_EXIT, 93
.equ STDOUT, 1

.equ NUMBUF_SIZE, 32
.equ STATE_SIZE, 40
.equ OUT_SIZE, 64

.section .bss
.balign 8
total_cases:
    .quad 0
failed_cases:
    .quad 0
numbuf:
    .space NUMBUF_SIZE
test_state:
    .space STATE_SIZE
test_out:
    .space OUT_SIZE

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
    mov x9, x0
pc_len:
    ldrb w2, [x0]
    cbz w2, pc_len_done
    add x0, x0, #1
    b pc_len
pc_len_done:
    sub x1, x0, x9
    mov x0, x9
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
    add w6, w6, #48
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
ce_true:
    mov x0, #1
    ret
ce_tail_equal:
    ldrb w5, [x3, x0]
    cbnz w5, ce_false
    b ce_true
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

.global test_reset_state
.type test_reset_state, %function
test_reset_state:
    adrp x0, test_state
    add x0, x0, :lo12:test_state
    b calc_reset

.global keys
.type keys, %function
keys:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    mov x19, x0
keys_loop:
    ldrb w0, [x19]
    cbz w0, keys_done
    sub w20, w0, #48
    cmp w20, #9
    b.hi keys_special
    mov x21, x20
    b keys_press
keys_special:
    cmp w0, #46
    b.eq keys_dot
    cmp w0, #61
    b.eq keys_equals
    cmp w0, #43
    b.eq keys_add
    cmp w0, #45
    b.eq keys_sub
    cmp w0, #42
    b.eq keys_mul
    cmp w0, #47
    b.eq keys_div
    cmp w0, #99
    b.eq keys_clear
    cmp w0, #126
    b.eq keys_sign
    mov x21, xzr
    b keys_press
keys_dot:
    mov x21, #10
    b keys_press
keys_equals:
    mov x21, #11
    b keys_press
keys_add:
    mov x21, #12
    b keys_press
keys_sub:
    mov x21, #13
    b keys_press
keys_mul:
    mov x21, #14
    b keys_press
keys_div:
    mov x21, #15
    b keys_press
keys_clear:
    mov x21, #16
    b keys_press
keys_sign:
    mov x21, #17
    b keys_press
keys_press:
    adrp x0, test_state
    add x0, x0, :lo12:test_state
    mov w1, w21
    bl calc_key
    add x19, x19, #1
    b keys_loop
keys_done:
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global expect
.type expect, %function
expect:
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    mov x19, x0
    mov x20, x1
    adrp x0, test_state
    add x0, x0, :lo12:test_state
    adrp x1, test_out
    add x1, x1, :lo12:test_out
    bl calc_format
    mov x0, x19
    adrp x1, test_out
    add x1, x1, :lo12:test_out
    mov x2, x20
    bl check
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret

.global run_case
.type run_case, %function
run_case:
    stp x29, x30, [sp, #-32]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    mov x19, x0
    ldr x0, [x19]
    bl print_cstr
    bl test_reset_state
    ldr x0, [x19, #8]
    bl keys
    ldr x0, [x19]
    ldr x1, [x19, #16]
    bl expect
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #32
    ret

.global calc_test_main
.type calc_test_main, %function
calc_test_main:
    stp x29, x30, [sp, #-32]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    bl framework_selftest
    adrp x0, test_table
    add x0, x0, :lo12:test_table
    mov x19, x0
table_next:
    ldr x0, [x19]
    cbz x0, table_done
    mov x0, x19
    bl run_case
    add x19, x19, #24
    b table_next
table_done:
    bl print_summary
    adrp x0, failed_cases
    add x0, x0, :lo12:failed_cases
    ldr x0, [x0]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #32
    ret

.section .rodata
.balign 8
t_reset:
    .asciz "reset shows zero"
t_digit:
    .asciz "a single digit is typed"
t_digits:
    .asciz "several digits are typed"
t_cap:
    .asciz "entry stops at twelve integer digits"
t_dot_start:
    .asciz "a leading dot starts a fresh entry"
t_dot_half:
    .asciz "a half is entered as a fraction"
t_dot_twice:
    .asciz "a second dot is ignored"
t_dot_shift:
    .asciz "a fifth fractional digit shifts the window"
t_clear:
    .asciz "clear returns to zero"
t_clear_after_digits:
    .asciz "clear after digits returns to zero"
t_mul:
    .asciz "twelve times twelve is a hundred and forty four"
t_decimal_sum:
    .asciz "a tenth plus a fifth is exactly three tenths"
t_sub_negative:
    .asciz "five minus eight is negative three"
t_sub_positive:
    .asciz "eight minus five is three"
t_neg_add:
    .asciz "negative eight plus five is negative three"
t_sign_toggle:
    .asciz "the sign key negates the typed value"
t_sign_zero:
    .asciz "negating zero stays zero"
t_chain:
    .asciz "operators chain with immediate execution"
t_chain_mid:
    .asciz "a pending operator shows the accumulated value"
t_repeat_equals:
    .asciz "a repeated equals repeats the last operation"
t_div_thirds:
    .asciz "one third rounds to four decimals"
t_div_two_thirds:
    .asciz "two thirds rounds up"
t_div_chain:
    .asciz "division chains left to right"
t_div_by_zero:
    .asciz "dividing by zero latches an error"
t_error_clears:
    .asciz "clear leaves the error state"
t_error_ignores:
    .asciz "digits are ignored while an error is latched"
t_overflow:
    .asciz "a product past the magnitude limit latches an error"
t_big_sum:
    .asciz "large magnitudes still add exactly"
k_reset:
    .asciz "c"
k_digit:
    .asciz "7"
k_digits:
    .asciz "1234"
k_cap:
    .asciz "1234567890123"
k_dot_start:
    .asciz ".5"
k_dot_half:
    .asciz "0.5"
k_dot_twice:
    .asciz "0.5.5"
k_dot_shift:
    .asciz "0.12345"
k_clear:
    .asciz "c"
k_clear_after_digits:
    .asciz "123c"
k_mul:
    .asciz "12*12="
k_decimal_sum:
    .asciz "0.1+0.2="
k_sub_negative:
    .asciz "5-8="
k_sub_positive:
    .asciz "8-5="
k_neg_add:
    .asciz "8~+5="
k_sign_toggle:
    .asciz "5~"
k_sign_zero:
    .asciz "0~"
k_chain:
    .asciz "2+3*4="
k_chain_mid:
    .asciz "2+3*"
k_repeat_equals:
    .asciz "2+3=="
k_div_thirds:
    .asciz "1/3="
k_div_two_thirds:
    .asciz "2/3="
k_div_chain:
    .asciz "100/5/2="
k_div_by_zero:
    .asciz "1/0="
k_error_clears:
    .asciz "1/0=c"
k_error_ignores:
    .asciz "1/0=5"
k_overflow:
    .asciz "999999999999*999999999999="
k_big_sum:
    .asciz "99999999999+1="
s_zero:
    .asciz "0"
s_seven:
    .asciz "7"
s_1234:
    .asciz "1234"
s_cap:
    .asciz "123456789012"
s_dot_five:
    .asciz "0.5"
s_leading_dot:
    .asciz "0.5"
s_dot_five_five:
    .asciz "0.55"
s_shift:
    .asciz "0.1235"
s_144:
    .asciz "144"
s_three_tenths:
    .asciz "0.3"
s_neg_three:
    .asciz "-3"
s_neg_five:
    .asciz "-5"
s_pos_three:
    .asciz "3"
s_20:
    .asciz "20"
s_five:
    .asciz "5"
s_8:
    .asciz "8"
s_one_third:
    .asciz "0.3333"
s_two_thirds:
    .asciz "0.6667"
s_10:
    .asciz "10"
s_error:
    .asciz "Error"
s_5:
    .asciz "5"
s_100000000000:
    .asciz "100000000000"
.balign 8
test_table:
    .quad t_reset
    .quad k_reset
    .quad s_zero
    .quad t_digit
    .quad k_digit
    .quad s_seven
    .quad t_digits
    .quad k_digits
    .quad s_1234
    .quad t_cap
    .quad k_cap
    .quad s_cap
    .quad t_dot_start
    .quad k_dot_start
    .quad s_leading_dot
    .quad t_dot_half
    .quad k_dot_half
    .quad s_dot_five
    .quad t_dot_twice
    .quad k_dot_twice
    .quad s_dot_five_five
    .quad t_dot_shift
    .quad k_dot_shift
    .quad s_shift
    .quad t_clear
    .quad k_clear
    .quad s_zero
    .quad t_clear_after_digits
    .quad k_clear_after_digits
    .quad s_zero
    .quad t_mul
    .quad k_mul
    .quad s_144
    .quad t_decimal_sum
    .quad k_decimal_sum
    .quad s_three_tenths
    .quad t_sub_negative
    .quad k_sub_negative
    .quad s_neg_three
    .quad t_sub_positive
    .quad k_sub_positive
    .quad s_pos_three
    .quad t_neg_add
    .quad k_neg_add
    .quad s_neg_three
    .quad t_sign_toggle
    .quad k_sign_toggle
    .quad s_neg_five
    .quad t_sign_zero
    .quad k_sign_zero
    .quad s_zero
    .quad t_chain_mid
    .quad k_chain_mid
    .quad s_five
    .quad t_chain
    .quad k_chain
    .quad s_20
    .quad t_repeat_equals
    .quad k_repeat_equals
    .quad s_8
    .quad t_div_thirds
    .quad k_div_thirds
    .quad s_one_third
    .quad t_div_two_thirds
    .quad k_div_two_thirds
    .quad s_two_thirds
    .quad t_div_chain
    .quad k_div_chain
    .quad s_10
    .quad t_div_by_zero
    .quad k_div_by_zero
    .quad s_error
    .quad t_error_clears
    .quad k_error_clears
    .quad s_zero
    .quad t_error_ignores
    .quad k_error_ignores
    .quad s_error
    .quad t_overflow
    .quad k_overflow
    .quad s_error
    .quad t_big_sum
    .quad k_big_sum
    .quad s_100000000000
    .quad 0
    .quad 0
    .quad 0
