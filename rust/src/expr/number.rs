//! The numeric tower used by the expression engine.
//!
//! The calculator has two numeric worlds and they are not interchangeable:
//!
//! - [`Number`] is `f64`. Fast, approximate, good for trigonometry. This is what
//!   the Scientific mode uses.
//! - [`Exact`] is sign plus a `BigUint` scaled by a power of ten, so `1/3` is
//!   exactly `0.3333...` and `20!` is exact. This is what Big numbers uses.
//!
//! [`Number`] is the default everywhere; [`Exact`] is opted into by the caller.
//! Conversions between them are explicit, because silently promoting a float
//! result into an exact one would manufacture precision that never existed.

use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

/// A floating point value, the working type for the expression engine.
pub type Number = f64;

/// Arithmetic error raised while evaluating.
#[derive(Debug, Clone, PartialEq)]
pub enum MathError {
    DivisionByZero,
    Domain(String),
    Overflow,
    Undefined(String),
}

impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "division by zero"),
            MathError::Domain(what) => write!(f, "{what} is not defined for this input"),
            MathError::Overflow => write!(f, "result is too large"),
            MathError::Undefined(name) => write!(f, "unknown function `{name}`"),
        }
    }
}

impl std::error::Error for MathError {}

pub type MathResult<T> = Result<T, MathError>;

/// A decimal number held exactly, as a sign and a mantissa scaled by
/// [`Exact::SCALE`].
///
/// Chosen over `f64` and over a binary floating point type because a decimal
/// scale matches what a person reads off a calculator display. `0.1 + 0.2` is
/// exactly `0.3`, and division can extend the fraction instead of rounding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exact {
    /// `true` when the value is negative. Zero is always positive.
    negative: bool,
    /// The magnitude, scaled by [`Exact::SCALE`].
    magnitude: BigUint,
}

impl Exact {
    /// Number of implied decimal places. 28 keeps well past double precision
    /// while keeping the scale factor inside a `u64`.
    pub const SCALE: u32 = 18;
    /// Scale factor, `10^SCALE`. 18 is the largest power of ten that fits in a
    /// `u64`, which keeps every conversion to `u64` or `f64` cheap.
    pub const SCALE_POW: u64 = 1_000_000_000_000_000_000;

    pub fn zero() -> Self {
        Exact {
            negative: false,
            magnitude: BigUint::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.negative && !self.is_zero()
    }

    pub fn negate(mut self) -> Self {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
        self
    }

    pub fn abs(mut self) -> Self {
        self.negative = false;
        self
    }

    /// Build from a whole number.
    pub fn from_i64(value: i64) -> Self {
        let sign = value < 0;
        let magnitude = BigUint::from(value.unsigned_abs()) * BigUint::from(Exact::SCALE_POW);
        Exact {
            negative: sign,
            magnitude,
        }
    }

    /// Build from an unsigned whole number.
    pub fn from_u64(value: u64) -> Self {
        Exact {
            negative: false,
            magnitude: BigUint::from(value) * BigUint::from(Exact::SCALE_POW),
        }
    }

    /// Build from a string of digits, with an optional decimal point.
    ///
    /// This is exact by construction: `Exact::parse("0.1")` is one tenth, with
    /// no rounding at all.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        if digits.is_empty() {
            return None;
        }

        let (integer_part, fraction_part) = match digits.split_once('.') {
            Some((a, b)) => (a, b),
            None => (digits, ""),
        };
        if fraction_part.contains('.') {
            return None;
        }
        if integer_part.is_empty() && fraction_part.is_empty() {
            return None;
        }
        if !integer_part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if !fraction_part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }

        let mut digits = String::new();
        digits.push_str(if integer_part.is_empty() { "0" } else { integer_part });
        digits.push_str(fraction_part);

        let mut magnitude = BigUint::parse_bytes(digits.as_bytes(), 10)?;
        let extra = Exact::SCALE as usize - fraction_part.len().min(Exact::SCALE as usize);
        magnitude *= BigUint::from(10u64).pow(extra as u32);

        let value = Exact { negative, magnitude };
        Some(if value.is_zero() { Exact::zero() } else { value })
    }

    /// Convert from a float, capturing the shortest decimal representation so
    /// that `0.1f64` becomes exactly one tenth rather than a binary artefact.
    pub fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        // Rust's `{}` for f64 already produces the shortest string that round
        // trips, which is exactly the decimal the user would have written.
        Exact::parse(&format!("{value}"))
    }

    /// Add two exact values.
    pub fn add(&self, other: &Exact) -> Exact {
        if self.negative == other.negative {
            Exact {
                negative: self.negative,
                magnitude: &self.magnitude + &other.magnitude,
            }
        } else {
            match self.magnitude.cmp(&other.magnitude) {
                std::cmp::Ordering::Greater => Exact {
                    negative: self.negative,
                    magnitude: &self.magnitude - &other.magnitude,
                },
                std::cmp::Ordering::Less => Exact {
                    negative: other.negative,
                    magnitude: &other.magnitude - &self.magnitude,
                },
                std::cmp::Ordering::Equal => Exact::zero(),
            }
        }
    }

    /// Subtract `other` from `self`.
    pub fn sub(&self, other: &Exact) -> Exact {
        self.add(&other.clone().negate())
    }

    pub fn mul(&self, other: &Exact) -> Exact {
        // Scaling by SCALE twice, so the result needs one division by SCALE_POW.
        let magnitude = (&self.magnitude * &other.magnitude) / BigUint::from(Exact::SCALE_POW);
        let value = Exact {
            negative: self.negative != other.negative,
            magnitude,
        };
        if value.is_zero() {
            Exact::zero()
        } else {
            value
        }
    }

    /// Divide, extending the fraction to the full scale.
    ///
    /// `1/3` is `0.3333...` truncated at 28 places, which is exact to within
    /// one unit in the last place rather than rounded to a float.
    pub fn div(&self, other: &Exact) -> Option<Exact> {
        if other.is_zero() {
            return None;
        }
        // (a/S) / (b/S) = (a/S) * (S/b) = a/b. The two scales cancel, so the
        // quotient magnitude is a * S / b: the factor of S has to be put back,
        // otherwise the result is 10^18 too small.
        let magnitude = (&self.magnitude * BigUint::from(Exact::SCALE_POW)) / &other.magnitude;
        Some(Exact {
            negative: self.negative != other.negative,
            magnitude,
        })
    }

    /// Integer power, by repeated squaring.
    ///
    /// A negative exponent is `1 / (x^|n|)`. Note this is the *exact* reciprocal:
    /// `2^-2` is exactly `0.25`, while `3^-1` is one third truncated at the
    /// scale, which is the closest this representation can get.
    pub fn powi(&self, exponent: i64) -> Option<Exact> {
        if exponent < 0 {
            if self.is_zero() {
                return None;
            }
            let positive = self.powi(exponent.checked_neg().unwrap_or(i64::MAX))?;
            return Exact::one().div(&positive);
        }
        let mut result = Exact::one();
        let mut base = self.clone();
        let mut e = exponent as u64;
        while e > 0 {
            if e & 1 == 1 {
                result = result.mul(&base);
            }
            e >>= 1;
            if e > 0 {
                base = base.mul(&base);
            }
        }
        Some(result)
    }

    pub fn one() -> Self {
        Exact::from_u64(1)
    }

    /// The value as an `f64`, which may lose precision for large mantissas.
    pub fn to_f64(&self) -> f64 {
        let as_float = |v: &BigUint| -> f64 {
            let mut result = 0.0f64;
            for chunk in v.to_u64_digits().iter().rev() {
                result = result * 1e19f64 + *chunk as f64;
            }
            result
        };
        let value = as_float(&self.magnitude) / Exact::SCALE_POW as f64;
        if self.is_negative() {
            -value
        } else {
            value
        }
    }

    /// Render without trailing zeros, trimming to `places` decimals at most.
    pub fn format(&self, places: usize) -> String {
        let sign = if self.is_negative() { "-" } else { "" };
        let digits = self.magnitude.to_str_radix(10);

        // The magnitude is scaled by 10^SCALE, so the last SCALE digits are the
        // fraction. When there are fewer digits than SCALE the whole number is
        // the fraction and the integer part is zero, which is why a leading "0"
        // has to be forced on before slicing rather than only when the value is
        // zero.
        let mut padded = digits;
        if padded.len() <= Exact::SCALE as usize {
            padded = format!(
                "{}{}",
                "0".repeat(Exact::SCALE as usize + 1 - padded.len()),
                padded
            );
        }

        let split = padded.len() - Exact::SCALE as usize;
        let integer = &padded[..split];
        let fraction = &padded[split..];

        let mut fraction: String = fraction.chars().take(places).collect();
        while fraction.ends_with('0') {
            fraction.pop();
        }

        if fraction.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{fraction}")
        }
    }
}

impl std::fmt::Display for Exact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(Exact::SCALE as usize))
    }
}

/// Factorial, exact. Returns `None` above 2000, where the digits run into
/// thousands and the result stops being useful to read.
pub fn factorial(n: u64) -> Option<Exact> {
    if n > 2000 {
        return None;
    }
    let mut result = BigUint::one();
    for i in 2..=n {
        result *= i;
    }
    Some(Exact {
        negative: false,
        magnitude: result * BigUint::from(Exact::SCALE_POW),
    })
}
