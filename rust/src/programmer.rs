//! Programmer mode: integers, multiple bases and bitwise operations.
//!
//! The arithmetic kernel has no concept of a number base, so this module works
//! on `u64` directly and is deliberately separate from the expression engine.
//! Values wrap at the word size, which is what a real programmer calculator
//! does, and two's complement is available for signed interpretation.

/// A number base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
}

impl Base {
    pub fn from_jni(value: i32) -> Base {
        match value {
            1 => Base::Hexadecimal,
            2 => Base::Octal,
            3 => Base::Binary,
            _ => Base::Decimal,
        }
    }

    pub fn to_jni(self) -> i32 {
        match self {
            Base::Decimal => 0,
            Base::Hexadecimal => 1,
            Base::Octal => 2,
            Base::Binary => 3,
        }
    }

    pub fn radix(self) -> u32 {
        match self {
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
            Base::Octal => 8,
            Base::Binary => 2,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Base::Decimal => "DEC",
            Base::Hexadecimal => "HEX",
            Base::Octal => "OCT",
            Base::Binary => "BIN",
        }
    }

    /// The digits this base can accept as input.
    pub fn accepts(self, ch: char) -> bool {
        let ch = ch.to_ascii_uppercase();
        match self {
            Base::Decimal => ch.is_ascii_digit(),
            Base::Hexadecimal => ch.is_ascii_hexdigit(),
            Base::Octal => ('0'..='7').contains(&ch),
            Base::Binary => ch == '0' || ch == '1',
        }
    }

    /// Render a value in this base, grouped for readability.
    pub fn format(self, value: u64) -> String {
        let text = match self {
            Base::Decimal => value.to_string(),
            Base::Hexadecimal => format!("{value:X}"),
            Base::Octal => format!("{value:o}"),
            Base::Binary => format!("{value:b}"),
        };
        if self == Base::Decimal {
            return text;
        }

        // Group long digit strings so a 64 bit value stays readable.
        let group = match self {
            Base::Hexadecimal => 4,
            Base::Octal => 11,
            Base::Binary => 4,
            Base::Decimal => 0,
        };
        if group == 0 || text.len() <= group {
            return text;
        }
        let digits: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len() + text.len() / group);
        for (index, ch) in digits.iter().enumerate() {
            if index > 0 && (digits.len() - index) % group == 0 {
                out.push(' ');
            }
            out.push(*ch);
        }
        out
    }

    /// Parse a value written in this base, ignoring spaces and separators.
    pub fn parse(self, text: &str) -> Option<u64> {
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '_')
            .collect();
        u64::from_str_radix(&cleaned, self.radix()).ok()
    }
}

/// The word size a programmer calculator works in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSize {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
}

impl WordSize {
    pub fn from_jni(value: i32) -> WordSize {
        match value {
            0 => WordSize::Bits8,
            1 => WordSize::Bits16,
            2 => WordSize::Bits32,
            _ => WordSize::Bits64,
        }
    }

    pub fn to_jni(self) -> i32 {
        match self {
            WordSize::Bits8 => 0,
            WordSize::Bits16 => 1,
            WordSize::Bits32 => 2,
            WordSize::Bits64 => 3,
        }
    }

    pub fn bits(self) -> u32 {
        match self {
            WordSize::Bits8 => 8,
            WordSize::Bits16 => 16,
            WordSize::Bits32 => 32,
            WordSize::Bits64 => 64,
        }
    }

    /// The mask that keeps a value inside this word.
    pub fn mask(self) -> u64 {
        match self {
            WordSize::Bits8 => 0xFF,
            WordSize::Bits16 => 0xFFFF,
            WordSize::Bits32 => 0xFFFF_FFFF,
            WordSize::Bits64 => u64::MAX,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            WordSize::Bits8 => "8",
            WordSize::Bits16 => "16",
            WordSize::Bits32 => "32",
            WordSize::Bits64 => "64",
        }
    }
}

/// A bitwise operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bitwise {
    And,
    Or,
    Xor,
    Not,
    ShiftLeft,
    ShiftRight,
    Modulo,
}

impl Bitwise {
    pub fn from_jni(value: i32) -> Bitwise {
        match value {
            0 => Bitwise::And,
            1 => Bitwise::Or,
            2 => Bitwise::Xor,
            3 => Bitwise::Not,
            4 => Bitwise::ShiftLeft,
            5 => Bitwise::ShiftRight,
            _ => Bitwise::Modulo,
        }
    }

    pub fn apply(self, a: u64, b: u64, word: WordSize) -> u64 {
        let raw = match self {
            Bitwise::And => a & b,
            Bitwise::Or => a | b,
            Bitwise::Xor => a ^ b,
            Bitwise::Not => !a,
            Bitwise::ShiftLeft => {
                // Shifts past the word are zero, not undefined.
                if b >= u64::from(word.bits()) {
                    0
                } else {
                    a << b
                }
            }
            Bitwise::ShiftRight => {
                if b >= u64::from(word.bits()) {
                    0
                } else {
                    a >> b
                }
            }
            Bitwise::Modulo => {
                if b == 0 {
                    0
                } else {
                    a % b
                }
            }
        };
        raw & word.mask()
    }
}

/// A programmer mode calculator.
#[derive(Debug, Clone)]
pub struct Programmer {
    pub value: u64,
    pub base: Base,
    pub word: WordSize,
    /// The last operand, so repeated bitwise operations reuse it.
    pub last_operand: u64,
}

impl Default for Programmer {
    fn default() -> Self {
        Self::new()
    }
}

impl Programmer {
    pub fn new() -> Self {
        Programmer {
            value: 0,
            base: Base::Decimal,
            word: WordSize::Bits64,
            last_operand: 0,
        }
    }

    pub fn clear(&mut self) {
        self.value = 0;
    }

    /// Append a digit, respecting the current base and word size.
    ///
     /// Returns false when the digit is not valid in this base, which is how the
    /// UI knows to ignore the key rather than corrupt the value.
    pub fn push_digit(&mut self, ch: char) -> bool {
        if !self.base.accepts(ch) {
            return false;
        }
        let radix = self.base.radix();
        let digit = ch.to_digit(radix).unwrap_or(0);
        // Detect overflow before it happens rather than wrapping silently.
        let next = self
            .value
            .checked_mul(u64::from(radix))
            .and_then(|scaled| scaled.checked_add(u64::from(digit)));
        match next {
            Some(value) if value <= self.word.mask() => {
                self.value = value;
                true
            }
            _ => false,
        }
    }

    /// Remove the most significant digit.
    pub fn backspace(&mut self) {
        self.value /= u64::from(self.base.radix());
    }

    /// Apply a bitwise operation against a literal operand.
    pub fn apply(&mut self, operation: Bitwise, operand: u64) {
        self.last_operand = operand;
        self.value = operation.apply(self.value, operand, self.word);
    }

    /// Repeat the last bitwise operation, the way a real programmer calculator
    /// repeats an arithmetic one.
    pub fn repeat(&mut self, operation: Bitwise) {
        self.value = operation.apply(self.value, self.last_operand, self.word);
    }

    /// The value as a signed two's complement number.
    pub fn signed(&self) -> i64 {
        match self.word {
            WordSize::Bits64 => self.value as i64,
            _ => {
                let bits = self.word.bits();
                let sign = 1u64 << (bits - 1);
                if self.value & sign != 0 {
                    (self.value as i64) - (1i64 << bits)
                } else {
                    self.value as i64
                }
            }
        }
    }

    /// The display text in the current base.
    pub fn display(&self) -> String {
        self.base.format(self.value)
    }

    /// Every base at once, which is what a programmer calculator shows.
    pub fn all_bases(&self) -> Vec<(Base, String)> {
        vec![
            (Base::Hexadecimal, Base::Hexadecimal.format(self.value)),
            (Base::Decimal, Base::Decimal.format(self.value)),
            (Base::Octal, Base::Octal.format(self.value)),
            (Base::Binary, Base::Binary.format(self.value)),
        ]
    }
}
