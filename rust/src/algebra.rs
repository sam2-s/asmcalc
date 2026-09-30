//! Matrices and complex numbers.
//!
//! Both are small fixed shapes on purpose: a calculator that can show you a
//! determinant needs a shape you can see, not an arbitrary dimension.

use std::fmt;

/// A dense matrix of `f64`, stored row major.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<f64>,
}

impl Matrix {
    /// A matrix of zeroes.
    pub fn zeros(rows: usize, cols: usize) -> Matrix {
        Matrix {
            rows,
            cols,
            cells: vec![0.0; rows * cols],
        }
    }

    /// A matrix from row major values. Returns `None` on a size mismatch.
    pub fn from_rows(rows: &[Vec<f64>]) -> Option<Matrix> {
        let row_count = rows.len();
        let col_count = rows.first()?.len();
        if col_count == 0 || rows.iter().any(|row| row.len() != col_count) {
            return None;
        }
        let cells = rows.iter().flat_map(|row| row.iter().copied()).collect();
        Some(Matrix {
            rows: row_count,
            cols: col_count,
            cells,
        })
    }

    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row < self.rows && col < self.cols {
            Some(self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Element-wise addition, which needs matching shapes.
    pub fn add(&self, other: &Matrix) -> Option<Matrix> {
        if self.shape() != other.shape() {
            return None;
        }
        Some(Matrix {
            rows: self.rows,
            cols: self.cols,
            cells: self
                .cells
                .iter()
                .zip(other.cells.iter())
                .map(|(a, b)| a + b)
                .collect(),
        })
    }

    /// Matrix product in row major order, which needs inner dimensions to match.
    pub fn mul(&self, other: &Matrix) -> Option<Matrix> {
        if self.cols != other.rows {
            return None;
        }
        let mut cells = vec![0.0; self.rows * other.cols];
        for row in 0..self.rows {
            for col in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.cells[row * self.cols + k] * other.cells[k * other.cols + col];
                }
                cells[row * other.cols + col] = sum;
            }
        }
        Some(Matrix {
            rows: self.rows,
            cols: other.cols,
            cells,
        })
    }

    /// Scalar multiplication.
    pub fn scale(&self, factor: f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            cells: self.cells.iter().map(|value| value * factor).collect(),
        }
    }

    /// Transpose.
    pub fn transpose(&self) -> Matrix {
        let mut cells = vec![0.0; self.cells.len()];
        for row in 0..self.rows {
            for col in 0..self.cols {
                cells[col * self.rows + row] = self.cells[row * self.cols + col];
            }
        }
        Matrix {
            rows: self.cols,
            cols: self.rows,
            cells,
        }
    }

    /// The determinant, by cofactor expansion.
    ///
    /// Recursive, which is fine at the sizes a calculator shows.
    pub fn determinant(&self) -> Option<f64> {
        match (self.rows, self.cols) {
            (1, 1) => return Some(self.cells[0]),
            (2, 2) => {
                return Some(
                    self.cells[0] * self.cells[3] - self.cells[1] * self.cells[2],
                )
            }
            (0, _) | (_, 0) => return None,
            _ => {}
        }
        if self.rows != self.cols {
            return None;
        }

        let mut total = 0.0;
        for col in 0..self.cols {
            let minor = self.minor(0, col);
            let minor_determinant = minor.determinant()?;
            let sign = if col % 2 == 0 { 1.0 } else { -1.0 };
            total += sign * self.cells[col] * minor_determinant;
        }
        Some(total)
    }

    /// The matrix with one row and one column removed.
    pub fn minor(&self, row: usize, col: usize) -> Matrix {
        let mut cells = Vec::with_capacity((self.rows - 1) * (self.cols - 1));
        for r in 0..self.rows {
            if r == row {
                continue;
            }
            for c in 0..self.cols {
                if c == col {
                    continue;
                }
                cells.push(self.cells[r * self.cols + c]);
            }
        }
        Matrix {
            rows: self.rows - 1,
            cols: self.cols - 1,
            cells,
        }
    }

    /// The inverse, or `None` when the matrix is singular.
    pub fn inverse(&self) -> Option<Matrix> {
        let determinant = self.determinant()?;
        if determinant.abs() < f64::EPSILON {
            return None;
        }

        // adj(A) is the transpose of the cofactor matrix.
        let mut adjugate = Matrix::zeros(self.cols, self.rows);
        for row in 0..self.rows {
            for col in 0..self.cols {
                let cofactor = self.minor(row, col).determinant()?;
                let sign = if (row + col) % 2 == 0 { 1.0 } else { -1.0 };
                adjugate.cells[col * self.rows + row] = sign * cofactor;
            }
        }
        Some(adjugate.scale(1.0 / determinant))
    }

    /// The trace, the sum of the diagonal.
    pub fn trace(&self) -> f64 {
        if self.rows != self.cols {
            return 0.0;
        }
        (0..self.rows)
            .map(|index| self.cells[index * self.cols + index])
            .sum()
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = self
            .cells
            .iter()
            .map(|value| format!("{value:.4}").len())
            .max()
            .unwrap_or(1)
            .max(3);
        for row in 0..self.rows {
            write!(f, "|")?;
            for col in 0..self.cols {
                write!(f, " {:>width$.4}", self.cells[row * self.cols + col])?;
            }
            writeln!(f, " |")?;
        }
        Ok(())
    }
}

/// A complex number in rectangular form.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Complex {
        Complex { re, im }
    }

    /// The complex quotient, guarded against a zero denominator.
    ///
    /// Named `quotient` rather than `div` because it cannot be a
    /// `std::ops::Div` implementation: there is no sensible value to return for
    /// a zero denominator, and the trait has no way to report that.
    pub fn quotient(self, other: Complex) -> Option<Complex> {
        let denominator = other.re * other.re + other.im * other.im;
        if denominator == 0.0 {
            return None;
        }
        Some(Complex::new(
            (self.re * other.re + self.im * other.im) / denominator,
            (self.im * other.re - self.re * other.im) / denominator,
        ))
    }

    /// The complex conjugate, which flips the sign of the imaginary part.
    pub fn conjugate(self) -> Complex {
        Complex::new(self.re, -self.im)
    }

    /// The modulus, written |z|.
    pub fn modulus(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// The argument, the angle from the positive real axis.
    pub fn argument(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// The conjugate, norm, cubed, which is the usual way to get a polar form.
    pub fn to_polar(self) -> Complex {
        let modulus = self.modulus();
        if modulus == 0.0 {
            return Complex::zero();
        }
        Complex::new(self.re / modulus, self.im / modulus)
    }

    pub fn zero() -> Complex {
        Complex::new(0.0, 0.0)
    }

    /// Parse `3+4i`, `-2i`, `5` or `(1+2i)`.
    pub fn parse(text: &str) -> Option<Complex> {
        let cleaned = text
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .replace(' ', "");
        if cleaned.is_empty() {
            return None;
        }

        if let Some(stripped) = cleaned.strip_suffix('i') {
            // Purely imaginary, or `a+bi`.
            if let Some(index) = find_split(stripped) {
                let (re_text, im_text) = stripped.split_at(index);
                // Keep the separator: without it "3-4" would parse the imaginary
                // part as positive.
                if im_text.is_empty() {
                    return None;
                }
                return Some(Complex::new(re_text.parse().ok()?, im_text.parse().ok()?));
            }
            let im: f64 = stripped.parse().ok()?;
            return Some(Complex::new(0.0, im));
        }

        Some(Complex::new(cleaned.parse().ok()?, 0.0))
    }
}

/// Find where the real and imaginary parts meet in `a+b`, `a-b` or just `a`.
fn find_split(text: &str) -> Option<usize> {
    // Skip a leading sign so `-2` is not read as an empty real part.
    let body = text.strip_prefix('-').map(str::len).unwrap_or(0);
    text[body..].find(['+', '-']).map(|index| body + index)
}

impl std::ops::Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
}

impl std::ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, other: Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }
}

/// The complex product: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
impl std::ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im == 0.0 {
            return write!(f, "{}", crate::expr::parser::format_number(self.re));
        }
        if self.re == 0.0 {
            return write!(f, "{}i", crate::expr::parser::format_number(self.im));
        }
        let sign = if self.im < 0.0 { "-" } else { "+" };
        write!(
            f,
            "{}{}{}i",
            crate::expr::parser::format_number(self.re),
            sign,
            crate::expr::parser::format_number(self.im.abs())
        )
    }
}
