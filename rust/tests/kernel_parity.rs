//! Parity tests: every case the assembly suite checks, replayed through the
//! Rust FFI wrapper.
//!
//! These are the same 29 cases as `app/src/test/asm/calc_tests.S`, expressed
//! against `FixedEngine`. Running them proves the FFI layer talks to the kernel
//! correctly; the assembly harness proves the kernel itself is correct.

use asmcalc::fixed::FixedEngine;

struct Case {
    keys: &'static str,
    expected: &'static str,
    what: &'static str,
}

const CASES: &[Case] = &[
    Case { keys: "c", expected: "0", what: "reset shows zero" },
    Case { keys: "7", expected: "7", what: "a single digit is typed" },
    Case { keys: "1234", expected: "1234", what: "several digits are typed" },
    Case { keys: "1234567890123", expected: "123456789012", what: "entry stops at twelve integer digits" },
    Case { keys: ".5", expected: "0.5", what: "a leading dot starts a fresh entry" },
    Case { keys: "0.5", expected: "0.5", what: "a half is entered as a fraction" },
    Case { keys: "0.5.5", expected: "0.55", what: "a second dot is ignored" },
    Case { keys: "0.12345", expected: "0.1235", what: "a fifth fractional digit shifts the window" },
    Case { keys: "c", expected: "0", what: "clear returns to zero" },
    Case { keys: "123c", expected: "0", what: "clear after digits returns to zero" },
    Case { keys: "12*12=", expected: "144", what: "twelve times twelve is a hundred and forty four" },
    Case { keys: "0.1+0.2=", expected: "0.3", what: "a tenth plus a fifth is exactly three tenths" },
    Case { keys: "5-8=", expected: "-3", what: "five minus eight is negative three" },
    Case { keys: "8-5=", expected: "3", what: "eight minus five is three" },
    Case { keys: "8~+5=", expected: "-3", what: "negative eight plus five is negative three" },
    Case { keys: "5~", expected: "-5", what: "the sign key negates the typed value" },
    Case { keys: "0~", expected: "0", what: "negating zero stays zero" },
    Case { keys: "2+3*", expected: "5", what: "a pending operator shows the accumulated value" },
    Case { keys: "2+3*4=", expected: "20", what: "operators chain with immediate execution" },
    Case { keys: "2+3==", expected: "8", what: "a repeated equals repeats the last operation" },
    Case { keys: "1/3=", expected: "0.3333", what: "one third rounds to four decimals" },
    Case { keys: "2/3=", expected: "0.6667", what: "two thirds rounds up" },
    Case { keys: "100/5/2=", expected: "10", what: "division chains left to right" },
    Case { keys: "1/0=", expected: "Error", what: "dividing by zero latches an error" },
    Case { keys: "1/0=c", expected: "0", what: "clear leaves the error state" },
    Case { keys: "1/0=5", expected: "Error", what: "digits are ignored while an error is latched" },
    Case { keys: "999999999999*999999999999=", expected: "Error", what: "a product past the magnitude limit latches an error" },
    Case { keys: "99999999999+1=", expected: "100000000000", what: "large magnitudes still add exactly" },
];

#[test]
fn ffi_reproduces_every_assembly_case() {
    let mut failures = Vec::new();

    for case in CASES {
        let mut engine = FixedEngine::new();
        engine.press_str(case.keys);
        let actual = engine.display();
        if actual != case.expected {
            failures.push(format!(
                "{}: keys {:?} expected {:?} got {:?}",
                case.what, case.keys, case.expected, actual
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        CASES.len(),
        failures.join("\n")
    );
}

#[test]
fn case_count_matches_the_assembly_suite() {
    // The assembly suite reports 29 cases: 28 calculator cases plus one
    // framework self-test that only exists there. These 28 are the calculator
    // cases, and the list is kept in step with app/src/test/asm/calc_tests.S.
    assert_eq!(CASES.len(), 28, "the assembly suite has 28 calculator cases");
}

#[test]
fn state_layout_matches_the_assembly_offsets() {
    use asmcalc::kernel::CalcState;
    use std::mem::{align_of, size_of};

    // These offsets come from app/src/main/asm/calc_state.h, which the C shim
    // asserts against the assembly constants with _Static_assert.
    let state = CalcState::zeroed();
    let base = &state as *const CalcState as usize;
    let offset = |field: *const u8| field as usize - base;

    assert_eq!(size_of::<CalcState>(), 40, "calc_state is 40 bytes");
    assert_eq!(align_of::<CalcState>(), 8, "calc_state is 8 byte aligned");
    assert_eq!(offset(&state.acc as *const u64 as *const u8), 0);
    assert_eq!(offset(&state.entry as *const u64 as *const u8), 8);
    assert_eq!(offset(&state.rhs as *const u64 as *const u8), 16);
    assert_eq!(offset(&state.acc_sign as *const u8 as *const u8), 24);
    assert_eq!(offset(&state.entry_sign as *const u8 as *const u8), 25);
    assert_eq!(offset(&state.rhs_sign as *const u8 as *const u8), 26);
    assert_eq!(offset(&state.dot as *const u8 as *const u8), 27);
    assert_eq!(offset(&state.int_digits as *const u8 as *const u8), 28);
    assert_eq!(offset(&state.frac_digits as *const u8 as *const u8), 29);
    assert_eq!(offset(&state.op as *const u8 as *const u8), 30);
    assert_eq!(offset(&state.last_op as *const u8 as *const u8), 31);
    assert_eq!(offset(&state.typing as *const u8 as *const u8), 32);
    assert_eq!(offset(&state.error as *const u8 as *const u8), 33);
}

#[test]
fn dividing_by_zero_latches_an_error() {
    let mut engine = FixedEngine::new();
    engine.press_str("1/0=");
    assert!(engine.is_error());
    assert_eq!(engine.display(), "Error");
}
