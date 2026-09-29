//! Expression engine tests. These run natively on the host, so they are fast.

use asmcalc::expr::number::{factorial, Exact, MathError};
use asmcalc::expr::{evaluate, evaluate_with, format_number, AngleMode};

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-9 * expected.abs().max(1.0)
}

fn value(input: &str) -> f64 {
    evaluate(input).unwrap_or_else(|e| panic!("{input:?} failed: {e}"))
}

#[test]
fn respects_precedence() {
    assert_eq!(value("2+3*4"), 14.0);
    assert_eq!(value("2*3+4"), 10.0);
    assert_eq!(value("2+3*4-6/2"), 11.0);
    assert_eq!(value("100/10/2"), 5.0, "division is left associative");
}

#[test]
fn power_is_right_associative() {
    assert_eq!(value("2^3^2"), 512.0, "2^(3^2), not (2^3)^2");
    assert_eq!(value("(2^3)^2"), 64.0);
}

#[test]
fn parentheses_nest() {
    assert_eq!(value("2*(3+4)^2"), 98.0);
    assert_eq!(value("((1+2)*(3+4))-5"), 16.0);
    assert_eq!(value("2*(3+(4-1))*2"), 24.0);
}

#[test]
fn unary_minus_binds_loosely() {
    assert_eq!(value("-3^2"), -9.0, "exponent beats unary minus");
    assert_eq!(value("(-3)^2"), 9.0);
    assert_eq!(value("3*-2"), -6.0);
    assert_eq!(value("--5"), 5.0);
}

#[test]
fn functions_and_constants() {
    assert!(close(value("sqrt(16)"), 4.0));
    assert!(close(value("abs(-7)"), 7.0));
    assert!(close(value("min(3,9)"), 3.0));
    assert!(close(value("max(3,9)"), 9.0));
    assert!(close(value("pow(2,10)"), 1024.0));
    assert!(close(value("hypot(3,4)"), 5.0));
    assert!(close(value("pi"), std::f64::consts::PI));
    assert!(close(value("e"), std::f64::consts::E));
}

#[test]
fn trig_follows_the_angle_mode() {
    // 30 degrees is pi/6, sin is 0.5.
    let deg = evaluate_with("sin(30)", AngleMode::Degrees).unwrap();
    assert!(close(deg, 0.5));
    let rad = evaluate_with("sin(pi/6)", AngleMode::Radians).unwrap();
    assert!(close(rad, 0.5));

    // Gradian: 100 grad is a right angle.
    let grad = evaluate_with("cos(100)", AngleMode::Gradians).unwrap();
    assert!(close(grad, 0.0));

    // The inverse goes the other way.
    let back = evaluate_with("asin(0.5)", AngleMode::Degrees).unwrap();
    assert!(close(back, 30.0));
    let back_rad = evaluate_with("asin(0.5)", AngleMode::Radians).unwrap();
    assert!(close(back_rad, std::f64::consts::FRAC_PI_6));
}

#[test]
fn inverse_trig_rejects_out_of_domain() {
    assert_eq!(evaluate("asin(2)"), Err(MathError::Domain("asin".into())));
    assert!(evaluate("ln(0)").is_err());
    assert!(evaluate("ln(-1)").is_err());
    assert!(evaluate("sqrt(-1)").is_err());
    assert!(evaluate("log(8,1)").is_err());
}

#[test]
fn factorial_and_rounding() {
    assert_eq!(value("5!"), 120.0);
    assert_eq!(value("0!"), 1.0);
    assert!(evaluate("2.5!").is_err(), "factorial needs a whole number");
    assert!(evaluate("(-1)!").is_err(), "factorial needs a non-negative");
    assert_eq!(value("floor(3.7)"), 3.0);
    assert_eq!(value("ceil(3.2)"), 4.0);
    assert_eq!(value("round(3.5)"), 4.0);
    assert_eq!(value("trunc(-3.7)"), -3.0);
}

#[test]
fn rejects_malformed_input() {
    for bad in [
        "2 +", "(1+2", "1+)", "2 3", "", "   ", "sin(", "unknown(1)", "1..2", "@", "log(2)",
    ] {
        assert!(evaluate(bad).is_err(), "{bad:?} should not parse");
    }
}

#[test]
fn arity_is_checked() {
    assert!(evaluate("min(1)").is_err());
    assert!(evaluate("pow(2)").is_err());
    assert!(evaluate("sin(1,2)").is_err());
}

#[test]
fn format_number_has_no_padding() {
    assert_eq!(format_number(144.0), "144");
    assert_eq!(format_number(0.5), "0.5");
    assert_eq!(format_number(0.1 + 0.2), "0.3");
    assert_eq!(format_number(-3.0), "-3");
    // The display shows ten decimals, so a third shows as 0.3333333333.
    assert_eq!(format_number(1.0 / 3.0), "0.3333333333");
}

// --- exact decimals -------------------------------------------------------

#[test]
fn exact_decimals_have_no_binary_drift() {
    let a = Exact::parse("0.1").unwrap();
    let b = Exact::parse("0.2").unwrap();
    let sum = a.add(&b);
    assert_eq!(sum.format(4), "0.3");
    // The float version famously is not 0.3.
    assert_ne!(0.1f64 + 0.2f64, 0.3f64);
}

#[test]
fn exact_division_extends_the_fraction() {
    let third = Exact::from_u64(1).div(&Exact::from_u64(3)).unwrap();
    assert_eq!(third.format(18), "0.333333333333333333");
    // Multiplying back gives 0.999..., which is the honest result of storing a
    // third in a finite number of digits.
    let back = third.mul(&Exact::from_u64(3));
    assert_eq!(back.format(18), "0.999999999999999999");
}

#[test]
fn exact_division_truncates_rather_than_rounding() {
    // The assembly kernel rounds 2/3 to 0.6667 at four decimals. Exact mode
    // deliberately does the opposite: it keeps the true digits and shows more
    // of them, because truncating a repeating fraction to something that looks
    // exact is the one thing exact mode exists to avoid.
    let value = Exact::from_u64(2).div(&Exact::from_u64(3)).unwrap();
    assert_eq!(value.format(4), "0.6666");
    assert_eq!(value.format(18), "0.666666666666666666");
}

#[test]
fn exact_arithmetic_basics() {
    let twelve = Exact::from_u64(12);
    assert_eq!(twelve.mul(&twelve).format(0), "144");
    assert_eq!(Exact::from_u64(5).sub(&Exact::from_u64(8)).format(0), "-3");
    assert_eq!(Exact::from_u64(8).sub(&Exact::from_u64(5)).format(0), "3");
    assert!(Exact::from_u64(1).div(&Exact::zero()).is_none());
    assert_eq!(Exact::from_u64(2).powi(-2).unwrap().format(4), "0.25");
    assert_eq!(Exact::from_i64(-7).abs().format(0), "7");
    assert_eq!(Exact::zero().negate().is_negative(), false, "no negative zero");
}

#[test]
fn exact_factorial_is_precise() {
    let twenty = factorial(20).unwrap();
    assert_eq!(
        twenty.format(0),
        "2432902008176640000",
        "20! is 2432902008176640000, which a double cannot hold"
    );
    // The same value as an f64 is famously wrong.
    assert_ne!(twenty.to_f64(), 2.43290200817664e18);
    assert!(factorial(2001).is_none());
}

#[test]
fn exact_rejects_nonsense() {
    assert!(Exact::parse("").is_none());
    assert!(Exact::parse("abc").is_none());
    assert!(Exact::parse("1.2.3").is_none());
    assert!(Exact::parse("-").is_none());
    assert!(Exact::from_f64(f64::NAN).is_none());
    assert!(Exact::from_f64(f64::INFINITY).is_none());
}

#[test]
fn exact_trims_trailing_zeros() {
    let value = Exact::parse("12.5000").unwrap();
    assert_eq!(value.format(10), "12.5");
    assert_eq!(Exact::parse("100").unwrap().format(4), "100");
}
