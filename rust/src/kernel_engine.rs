//! The arithmetic kernel, as a trait.
//!
//! On Android this is backed by the AArch64 assembly in `app/src/main/asm`. On
//! the host, where AArch64 code cannot run, a faithful Rust transcription stands
//! in. The two are held to the same 28 case suite, so the reference cannot
//! quietly drift from the assembly.

use crate::kernel::CalcState;

/// A calculator that can be driven key by key.
pub trait KeyEngine {
    fn press(&mut self, keycode: i32);
    fn display(&mut self) -> String;
    fn is_error(&self) -> bool;
}

/// The real kernel: AArch64 assembly, reached through the C ABI.
pub struct AsmEngine {
    state: CalcState,
    buffer: [u8; crate::kernel::FORMAT_BUFFER_SIZE],
}

impl AsmEngine {
    pub fn new() -> Self {
        let mut state = CalcState::zeroed();
        // SAFETY: `calc_reset` writes only the state it is handed, and `state`
        // is a live correctly sized local.
        unsafe { crate::kernel::calc_reset(&mut state) };
        AsmEngine {
            state,
            buffer: [0u8; crate::kernel::FORMAT_BUFFER_SIZE],
        }
    }
}

impl Default for AsmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEngine for AsmEngine {
    fn press(&mut self, keycode: i32) {
        // SAFETY: `calc_key` touches only the state it is given, which outlives
        // the call.
        unsafe { crate::kernel::calc_key(&mut self.state, keycode) };
    }

    fn display(&mut self) -> String {
        let length = {
            // SAFETY: the kernel writes at most FORMAT_BUFFER_SIZE bytes into
            // the buffer, which is exactly that size and live.
            let length =
                unsafe { crate::kernel::calc_format(&mut self.state, self.buffer.as_mut_ptr()) };
            length.clamp(0, crate::kernel::FORMAT_BUFFER_SIZE as i32) as usize
        };
        String::from_utf8_lossy(&self.buffer[..length]).into_owned()
    }

    fn is_error(&self) -> bool {
        self.state.error != 0
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub use host::ReferenceEngine as DefaultEngine;
#[cfg(target_arch = "aarch64")]
pub use AsmEngine as DefaultEngine;

/// A host stand in for the assembly kernel, so the layers above can be unit
/// tested natively. It mirrors the assembly semantics: sign-magnitude fixed
/// point, four decimals, immediate operator chaining.
#[cfg(not(target_arch = "aarch64"))]
pub mod host {
    use super::KeyEngine;
    use crate::kernel::*;

    const SCALE: i64 = 10000;
    const FRAC_MAX: u32 = 4;
    const INT_MAX: u32 = 12;

    pub struct ReferenceEngine {
        acc: i64,
        entry: i64,
        rhs: i64,
        acc_sign: bool,
        entry_sign: bool,
        rhs_sign: bool,
        dot: bool,
        int_digits: u32,
        frac_digits: u32,
        op: i32,
        last_op: i32,
        typing: bool,
        error: bool,
    }

    impl ReferenceEngine {
        pub fn new() -> Self {
            ReferenceEngine {
                acc: 0,
                entry: 0,
                rhs: 0,
                acc_sign: false,
                entry_sign: false,
                rhs_sign: false,
                dot: false,
                int_digits: 0,
                frac_digits: 0,
                op: 0,
                last_op: 0,
                typing: false,
                error: false,
            }
        }

        fn scale(&self, digits: i64) -> i64 {
            let mut value = digits;
            for _ in 0..(FRAC_MAX.saturating_sub(self.frac_digits)) {
                value = value.saturating_mul(10);
            }
            value
        }

        fn apply(&mut self, op: i32, a: i64, a_neg: bool, b: i64, b_neg: bool) -> (i64, bool) {
            match op {
                1 | 2 => {
                    let b_neg = if op == 2 { !b_neg } else { b_neg };
                    if a_neg == b_neg {
                        match a.checked_add(b) {
                            Some(sum) => (sum, a_neg),
                            None => {
                                self.error = true;
                                (0, false)
                            }
                        }
                    } else {
                        let (magnitude, sign) = if a >= b { (a - b, a_neg) } else { (b - a, b_neg) };
                        (magnitude, sign)
                    }
                }
                3 => {
                    // Fixed point: the product carries two scales, so one is
                    // divided out. The assembly does the same with umulh to
                    // catch the overflow.
                    match a.checked_mul(b) {
                        Some(product) => (product / SCALE, a_neg != b_neg),
                        None => {
                            self.error = true;
                            (0, false)
                        }
                    }
                }
                _ => {
                    if b == 0 {
                        self.error = true;
                        return (0, false);
                    }
                    let quotient = a / b;
                    let remainder = a % b;
                    let rounded = if remainder * 2 >= b { quotient + 1 } else { quotient };
                    (rounded, a_neg != b_neg)
                }
            }
        }

        fn commit_entry(&mut self) {
            if self.typing {
                if self.op != 0 {
                    // The typed entry is a raw digit accumulator and has to be
                    // brought onto the fixed point scale first, exactly as the
                    // assembly does with scale_entry.
                    let entry = self.scale(self.entry);
                    let (value, sign) = self.apply(self.op, self.acc, self.acc_sign, entry, self.entry_sign);
                    if !self.error {
                        self.acc = value;
                        self.acc_sign = sign;
                    }
                } else {
                    self.acc = self.scale(self.entry);
                    self.acc_sign = self.entry_sign;
                }
                self.clear_entry();
            }
        }

        fn clear_entry(&mut self) {
            self.typing = false;
            self.dot = false;
            self.entry_sign = false;
            self.frac_digits = 0;
            self.int_digits = 1;
            self.entry = 0;
        }
    }

    impl Default for ReferenceEngine {
        fn default() -> Self {
            Self::new()
        }
    }

    impl KeyEngine for ReferenceEngine {
        fn press(&mut self, keycode: i32) {
            if keycode == KEY_CLEAR {
                *self = ReferenceEngine::new();
                return;
            }
            if self.error {
                return;
            }

            if (0..=9).contains(&keycode) {
                self.digit(keycode as i64);
            } else {
                match keycode {
                    KEY_DOT => self.dot(),
                    KEY_ADD => self.operator(1),
                    KEY_SUB => self.operator(2),
                    KEY_MUL => self.operator(3),
                    KEY_DIV => self.operator(4),
                    KEY_SIGN => {
                        if self.typing {
                            self.entry_sign = !self.entry_sign;
                        } else {
                            self.acc_sign = !self.acc_sign;
                        }
                    }
                    KEY_EQUALS => self.equals(),
                    _ => {}
                }
            }
        }

        fn display(&mut self) -> String {
            if self.error {
                return "Error".to_string();
            }
            let (value, negative) = if self.typing {
                (self.scale(self.entry), self.entry_sign)
            } else {
                (self.acc, self.acc_sign)
            };

            let mut text = String::new();
            if negative && value != 0 {
                text.push('-');
            }
            let integer = value / SCALE;
            let fraction = value % SCALE;
            text.push_str(&integer.to_string());
            if fraction != 0 {
                let mut digits = format!("{fraction:04}");
                while digits.ends_with('0') {
                    digits.pop();
                }
                text.push('.');
                text.push_str(&digits);
            }
            text
        }

        fn is_error(&self) -> bool {
            self.error
        }
    }

    impl ReferenceEngine {
        fn digit(&mut self, digit: i64) {
            if !self.typing {
                self.entry = digit;
                self.typing = true;
                self.dot = false;
                self.int_digits = 1;
                self.frac_digits = 0;
                self.entry_sign = false;
                return;
            }
            if self.dot {
                if self.frac_digits < FRAC_MAX {
                    self.entry = self.entry * 10 + digit;
                    self.frac_digits += 1;
                } else {
                    self.entry = (self.entry * 10 + digit + 5) / 10;
                }
            } else if self.int_digits < INT_MAX {
                self.entry = self.entry * 10 + digit;
                self.int_digits += 1;
            }
        }

        fn dot(&mut self) {
            if !self.typing {
                self.entry = 0;
                self.typing = true;
                self.dot = true;
                self.int_digits = 1;
                self.frac_digits = 0;
                self.entry_sign = false;
            } else if !self.dot {
                self.dot = true;
            }
        }

        fn operator(&mut self, op: i32) {
            self.commit_entry();
            self.op = op;
        }

        fn equals(&mut self) {
            if self.typing {
                self.rhs = self.scale(self.entry);
                self.rhs_sign = self.entry_sign;
                if self.op != 0 {
                    let op = self.op;
                    let (value, sign) = self.apply(op, self.acc, self.acc_sign, self.rhs, self.rhs_sign);
                    if !self.error {
                        self.acc = value;
                        self.acc_sign = sign;
                        self.last_op = op;
                    }
                } else {
                    self.acc = self.rhs;
                    self.acc_sign = self.rhs_sign;
                }
                self.clear_entry();
            } else if self.last_op != 0 {
                let op = self.last_op;
                let (value, sign) = self.apply(op, self.acc, self.acc_sign, self.rhs, self.rhs_sign);
                if !self.error {
                    self.acc = value;
                    self.acc_sign = sign;
                }
            }
            self.op = 0;
        }
    }
}
