//! Programmer mode, unit conversion, matrices and complex numbers.

use asmcalc::algebra::{Complex, Matrix};
use asmcalc::programmer::{Base, Bitwise, Programmer, WordSize};
use asmcalc::units::{self, Category};

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-9 * expected.abs().max(1.0)
}

// --- programmer mode ------------------------------------------------------

#[test]
fn digits_follow_the_base() {
    let mut programmer = Programmer::new();
    programmer.base = Base::Binary;
    assert!(programmer.push_digit('1'));
    assert!(programmer.push_digit('0'));
    assert_eq!(programmer.value, 0b10);
    assert!(!programmer.push_digit('2'), "2 is not a binary digit");
    assert!(!programmer.push_digit('F'), "F is not a binary digit");
}

#[test]
fn hex_accepts_every_digit() {
    let mut programmer = Programmer::new();
    programmer.base = Base::Hexadecimal;
    for ch in "DEADBEEF".chars() {
        assert!(programmer.push_digit(ch), "{ch} should be a hex digit");
    }
    assert_eq!(programmer.value, 0xDEADBEEF);
    assert_eq!(programmer.display(), "DEAD BEEF", "long hex is grouped for reading");
}

#[test]
fn formatting_groups_long_values() {
    assert_eq!(Base::Binary.format(0b1010_1010), "1010 1010");
    assert_eq!(Base::Hexadecimal.format(0xDEAD_BEEF), "DEAD BEEF");
    assert_eq!(Base::Decimal.format(1234567), "1234567", "decimal is never grouped");
}

#[test]
fn word_size_masks_overflow() {
    let mut programmer = Programmer::new();
    programmer.base = Base::Hexadecimal;
    programmer.word = WordSize::Bits8;
    // 0xFF is the largest 8 bit value; one more must be refused, not wrapped.
    for ch in "FF".chars() {
        assert!(programmer.push_digit(ch));
    }
    assert_eq!(programmer.value, 255);
    assert!(!programmer.push_digit('F'), "256 does not fit in 8 bits");

    // A fresh engine: the previous block left value at 255, so reusing it would
    // overflow the very first push.
    let mut wide = Programmer::new();
    wide.base = Base::Hexadecimal;
    wide.word = WordSize::Bits16;
    for ch in "FFFF".chars() {
        assert!(wide.push_digit(ch), "{ch} should fit in 16 bits");
    }
    assert_eq!(wide.value, 0xFFFF);
}

#[test]
fn word_size_masks_results() {
    let mut programmer = Programmer::new();
    programmer.word = WordSize::Bits8;
    programmer.value = 200;
    programmer.value = 200;
    programmer.apply(Bitwise::And, 0xFF);
    assert_eq!(programmer.value, 200, "200 fits in 8 bits");

    let mut masked = Programmer::new();
    masked.word = WordSize::Bits8;
    masked.value = 200;
    masked.value &= masked.word.mask();
    assert_eq!(masked.value, 200);
}

#[test]
fn bitwise_operations() {
    let mut programmer = Programmer::new();
    programmer.value = 0b1100;
    programmer.apply(Bitwise::And, 0b1010);
    assert_eq!(programmer.value, 0b1000);

    programmer.value = 0b1100;
    programmer.apply(Bitwise::Or, 0b1010);
    assert_eq!(programmer.value, 0b1110);

    programmer.value = 0b1100;
    programmer.apply(Bitwise::Xor, 0b1010);
    assert_eq!(programmer.value, 0b0110);

    programmer.value = 0;
    programmer.apply(Bitwise::Not, 0);
    assert_eq!(programmer.value, u64::MAX, "not zero is all ones");
}

#[test]
fn shifts_respect_the_word() {
    let mut programmer = Programmer::new();
    programmer.word = WordSize::Bits8;
    programmer.value = 0b0000_0001;
    programmer.apply(Bitwise::ShiftLeft, 4);
    assert_eq!(programmer.value, 0b0001_0000);

    // Shifting by more than the word is zero, not undefined.
    programmer.value = 0xFF;
    programmer.apply(Bitwise::ShiftLeft, 8);
    assert_eq!(programmer.value, 0);
    programmer.value = 0xFF;
    programmer.apply(Bitwise::ShiftRight, 64);
    assert_eq!(programmer.value, 0);
}

#[test]
fn twos_complement_interpretation() {
    let mut programmer = Programmer::new();

    programmer.word = WordSize::Bits8;
    programmer.value = 0xFF;
    assert_eq!(programmer.signed(), -1);
    programmer.value = 0x80;
    assert_eq!(programmer.signed(), -128);
    programmer.value = 0x7F;
    assert_eq!(programmer.signed(), 127);

    programmer.word = WordSize::Bits32;
    programmer.value = 0xFFFF_FFFF;
    assert_eq!(programmer.signed(), -1);
    programmer.value = 0x8000_0000;
    assert_eq!(programmer.signed(), -2_147_483_648);
}

#[test]
fn all_bases_agree() {
    let mut programmer = Programmer::new();
    programmer.value = 255;
    let rendered: Vec<(Base, String)> = programmer.all_bases();
    for (base, text) in rendered {
        assert_eq!(
            base.parse(&text),
            Some(255),
            "{:?} rendered {text:?} did not round trip",
            base
        );
    }
}

#[test]
fn backspace_removes_a_digit() {
    let mut programmer = Programmer::new();
    for ch in "1234".chars() {
        programmer.push_digit(ch);
    }
    assert_eq!(programmer.value, 1234);
    programmer.backspace();
    assert_eq!(programmer.value, 123);
}

// --- unit conversion ------------------------------------------------------

#[test]
fn parses_a_conversion_request() {
    let request = units::parse_request("5 km to miles").unwrap();
    assert_eq!(request.value, 5.0);
    assert_eq!(request.from, "km");
    assert_eq!(request.to, "miles");

    assert!(units::parse_request("5km in mi").is_some());
    assert!(units::parse_request("5 km -> mi").is_some());
    assert!(units::parse_request("km to mi").is_none(), "a value is required");
    assert!(units::parse_request("5 km to").is_none());
}

#[test]
fn converts_length() {
    let (_, miles) = units::evaluate("5 km to miles").unwrap();
    assert!(close(miles, 3.106_855_961_18));

    let (_, feet) = units::evaluate("1 m to ft").unwrap();
    assert!(close(feet, 3.280_839_895));

    let (_, inches) = units::evaluate("1 ft to in").unwrap();
    assert!(close(inches, 12.0));
}

#[test]
fn converts_mass() {
    let (_, pounds) = units::evaluate("1 kg to lb").unwrap();
    assert!(close(pounds, 2.204_622_62));
    let (_, grams) = units::evaluate("1 lb to g").unwrap();
    assert!(close(grams, 453.592_37));
}

#[test]
fn temperature_handles_offsets() {
    // This is the case a single factor cannot express.
    let (_, fahrenheit) = units::evaluate("100 C to F").unwrap();
    assert!(close(fahrenheit, 212.0));
    let (_, celsius) = units::evaluate("32 F to C").unwrap();
    assert!(close(celsius, 0.0));
    let (_, kelvin) = units::evaluate("0 C to K").unwrap();
    assert!(close(kelvin, 273.15));
    let (_, round) = units::evaluate("-40 C to F").unwrap();
    assert!(close(round, -40.0), "-40 is where the scales meet");
}

#[test]
fn converts_the_remaining_categories() {
    for (text, expected) in [
        ("1 m2 to ft2", 10.763_910_417),
        ("1 l to ml", 1000.0),
        ("1 h to min", 60.0),
        ("1 km/h to m/s", 0.277_777_777_777_777_8),
        ("1 GiB to MiB", 1024.0),
    ] {
        let (_, value) = units::evaluate(text).unwrap();
        assert!(close(value, expected), "{text} gave {value}");
    }
}

#[test]
fn rejects_nonsense_conversions() {
    assert!(units::evaluate("5 km to kg").is_err(), "different categories");
    assert!(units::evaluate("5 parsecs to m").is_err(), "unknown unit");
    assert!(units::evaluate("not a conversion").is_err());
    assert!(units::factor(Category::Length, "furlong").is_none());
}

#[test]
fn every_category_has_units() {
    for category in [
        Category::Length,
        Category::Mass,
        Category::Temperature,
        Category::Area,
        Category::Volume,
        Category::Time,
        Category::Speed,
        Category::Data,
    ] {
        assert!(!category.units().is_empty(), "{} has no units", category.name());
        assert!(Category::from_name(category.name()).is_some());
    }
}

// --- matrices -------------------------------------------------------------

fn identity(n: usize) -> Matrix {
    let mut rows = vec![vec![0.0; n]; n];
    for (index, row) in rows.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    Matrix::from_rows(&rows).unwrap()
}

#[test]
fn matrix_addition_needs_matching_shapes() {
    let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
    let b = Matrix::from_rows(&[vec![10.0, 20.0], vec![30.0, 40.0]]).unwrap();
    let sum = a.add(&b).unwrap();
    assert_eq!(sum.get(0, 0), Some(11.0));
    assert_eq!(sum.get(1, 1), Some(44.0));

    let c = Matrix::from_rows(&[vec![1.0, 2.0, 3.0]]).unwrap();
    assert!(a.add(&c).is_none(), "shape mismatch");
}

#[test]
fn matrix_multiplication() {
    let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
    let b = Matrix::from_rows(&[vec![5.0, 6.0], vec![7.0, 8.0]]).unwrap();
    let product = a.mul(&b).unwrap();
    assert_eq!(product.get(0, 0), Some(19.0));
    assert_eq!(product.get(0, 1), Some(22.0));
    assert_eq!(product.get(1, 0), Some(43.0));
    assert_eq!(product.get(1, 1), Some(50.0));
}

#[test]
fn determinant_of_small_matrices() {
    let a = Matrix::from_rows(&[vec![4.0, 7.0], vec![2.0, 6.0]]).unwrap();
    assert!(close(a.determinant().unwrap(), 10.0));

    let b = Matrix::from_rows(&[vec![1.0, 0.0], vec![0.0, 1.0]]).unwrap();
    assert!(close(b.determinant().unwrap(), 1.0));

    // 3x3 by cofactor expansion: 1(9-2) - 2(6-3) + 3(4-2) = 7 - 6 + 6 = 7
    let c = Matrix::from_rows(&[
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ])
    .unwrap();
    assert!(close(c.determinant().unwrap(), 0.0), "this matrix is singular");

    let d = Matrix::from_rows(&[
        vec![1.0, 2.0, 3.0],
        vec![0.0, 1.0, 4.0],
        vec![5.0, 6.0, 0.0],
    ])
    .unwrap();
    assert!(close(d.determinant().unwrap(), 1.0));

    // A rectangular matrix has no determinant.
    let rectangular = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
    assert!(rectangular.determinant().is_none());
}

#[test]
fn inverse_undoes_the_original() {
    let a = Matrix::from_rows(&[vec![4.0, 7.0], vec![2.0, 6.0]]).unwrap();
    let inverse = a.inverse().unwrap();
    // (4 7)(2 6) has determinant 10, so the inverse is (6 -7)(-2 4)/10
    assert!(close(inverse.get(0, 0).unwrap(), 0.6));
    assert!(close(inverse.get(0, 1).unwrap(), -0.7));
    assert!(close(inverse.get(1, 0).unwrap(), -0.2));
    assert!(close(inverse.get(1, 1).unwrap(), 0.4));

    let product = a.mul(&inverse).unwrap();
    assert!(close(product.get(0, 0).unwrap(), 1.0));
    assert!(close(product.get(0, 1).unwrap(), 0.0));
    assert!(close(product.get(1, 1).unwrap(), 1.0));
}

#[test]
fn singular_matrices_have_no_inverse() {
    let singular = Matrix::from_rows(&[vec![1.0, 2.0], vec![2.0, 4.0]]).unwrap();
    assert!(singular.inverse().is_none());
    assert!(Matrix::zeros(2, 2).inverse().is_none());
    assert!(identity(3).inverse().is_some());
}

#[test]
fn transpose_and_trace() {
    let a = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
    let t = a.transpose();
    assert_eq!(t.shape(), (3, 2));
    assert_eq!(t.get(2, 0), Some(3.0));
    assert!(close(a.trace(), 0.0), "rectangular trace is zero");
    assert!(close(identity(3).trace(), 3.0));
}

// --- complex numbers ------------------------------------------------------

#[test]
fn complex_arithmetic() {
    let a = Complex::new(3.0, 4.0);
    let b = Complex::new(1.0, -2.0);

    assert_eq!(a + b, Complex::new(4.0, 2.0));
    assert_eq!(a - b, Complex::new(2.0, 6.0));
    // (3+4i)(1-2i) = 3-6i+4i-8i^2 = 11-2i
    assert_eq!(a * b, Complex::new(11.0, -2.0));
    assert_eq!(a.conjugate(), Complex::new(3.0, -4.0));
    assert!(close(a.modulus(), 5.0), "3-4-5");
    assert!(a.quotient(Complex::zero()).is_none());
}

#[test]
fn complex_division() {
    let a = Complex::new(3.0, 4.0);
    let b = Complex::new(1.0, 2.0);
    let quotient = a.quotient(b).unwrap();
    // (3+4i)/(1+2i) = (3+4i)(1-2i)/5 = (11-2i)/5
    assert!(close(quotient.re, 2.2));
    assert!(close(quotient.im, -0.4));

    // Dividing the original by the quotient returns the divisor, not the
    // original: (3+4i) / (2.2-0.4i) = 1+2i.
    let divisor = a.quotient(quotient).unwrap();
    assert!(close(divisor.re, 1.0));
    assert!(close(divisor.im, 2.0));

    // And the quotient times the divisor is the dividend.
    let product = quotient * b;
    assert!(close(product.re, 3.0));
    assert!(close(product.im, 4.0));
}

#[test]
fn complex_parsing() {
    assert_eq!(Complex::parse("3+4i"), Some(Complex::new(3.0, 4.0)));
    assert_eq!(Complex::parse("3-4i"), Some(Complex::new(3.0, -4.0)));
    assert_eq!(Complex::parse("5"), Some(Complex::new(5.0, 0.0)));
    assert_eq!(Complex::parse("2i"), Some(Complex::new(0.0, 2.0)));
    assert_eq!(Complex::parse("-2i"), Some(Complex::new(0.0, -2.0)));
    assert_eq!(Complex::parse("(1+2i)"), Some(Complex::new(1.0, 2.0)));
    assert_eq!(Complex::parse("nonsense"), None);
    assert_eq!(Complex::parse(""), None);
}

#[test]
fn complex_display() {
    assert_eq!(Complex::new(3.0, 4.0).to_string(), "3+4i");
    assert_eq!(Complex::new(3.0, -4.0).to_string(), "3-4i");
    assert_eq!(Complex::new(3.0, 0.0).to_string(), "3");
    assert_eq!(Complex::new(0.0, 4.0).to_string(), "4i");
}

#[test]
fn complex_polar_form() {
    let z = Complex::new(3.0, 4.0);
    let unit = z.to_polar();
    assert!(close(unit.modulus(), 1.0));
    assert!(close(unit.argument(), 0.927_295_218_0));
    assert_eq!(Complex::zero().to_polar(), Complex::zero());
}
