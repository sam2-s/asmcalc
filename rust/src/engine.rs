//! The calculator session: one object the UI talks to, spanning both engines.
//!
//! - [`Engine::Fixed`] replays keycodes through the AArch64 assembly kernel.
//! - [`Engine::Scientific`] evaluates expressions through the Rust engine.
//! - [`Engine::Programmer`] works in integer bases with bitwise operations.
//!
//! Memory registers and the history tape live here too, because both the
//! assembly and the expression engine have to be able to reach them.

use crate::expr::number::Exact;
use crate::expr::parser::{self, AngleMode};
use crate::kernel_engine::{DefaultEngine, KeyEngine};

/// Which calculator is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// The assembly kernel, in 4 decimal places.
    Fixed,
    /// Expressions and transcendentals, in f64.
    Scientific,
    /// Integers with bitwise operations and multiple bases.
    Programmer,
}

/// A memory register.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Register {
    pub value: f64,
    pub used: bool,
}

impl Register {
    pub const EMPTY: Register = Register {
        value: 0.0,
        used: false,
    };
}

/// One line of the history tape.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
}

/// Everything a session can show, in one snapshot.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub display: String,
    pub expression: String,
    pub angle_mode: &'static str,
    pub engine: Engine,
    pub history: Vec<HistoryEntry>,
    pub registers: [Register; 4],
}

/// A live calculator session.
pub struct Calculator {
    fixed: DefaultEngine,
    engine: Engine,
    angles: AngleMode,
    expression: String,
    display: String,
    history: Vec<HistoryEntry>,
    registers: [Register; 4],
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            fixed: DefaultEngine::new(),
            engine: Engine::Fixed,
            angles: AngleMode::Degrees,
            expression: String::new(),
            display: "0".to_string(),
            history: Vec::new(),
            registers: [Register::EMPTY; 4],
        }
    }

    pub fn engine(&self) -> Engine {
        self.engine
    }

    pub fn set_engine(&mut self, engine: Engine) {
        self.engine = engine;
        self.expression.clear();
    }

    pub fn angles(&self) -> AngleMode {
        self.angles
    }

    pub fn set_angles(&mut self, mode: AngleMode) {
        self.angles = mode;
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn history(&self) -> &[HistoryEntry] {
        &self.history
    }

    pub fn registers(&self) -> &[Register; 4] {
        &self.registers
    }

    /// Send a keycode to whichever engine is active.
    pub fn press(&mut self, keycode: i32) {
        match self.engine {
            Engine::Fixed => {
                self.fixed.press(keycode);
                self.display = self.fixed.display();
            }
            Engine::Scientific | Engine::Programmer => self.press_scientific(keycode),
        }
    }

    /// Append a character to the expression buffer and evaluate as we go.
    fn press_scientific(&mut self, keycode: i32) {
        let ch = match keycode {
            crate::kernel::KEY_CLEAR => {
                self.expression.clear();
                self.display = "0".to_string();
                return;
            }
            crate::kernel::KEY_EQUALS => {
                self.evaluate_expression();
                return;
            }
            crate::kernel::KEY_ADD => '+',
            crate::kernel::KEY_SUB => '-',
            crate::kernel::KEY_MUL => '*',
            crate::kernel::KEY_DIV => '/',
            crate::kernel::KEY_DOT => '.',
            crate::kernel::KEY_SIGN => '-',
            _ => {
                // Digits 0-9 map straight through.
                if (0..=9).contains(&keycode) {
                    char::from(b'0' + keycode as u8)
                } else {
                    return;
                }
            }
        };

        // `+/-` negates rather than appending, so a leading minus or a
        // substitution in place of an existing operator.
        if ch == '-' && keycode == crate::kernel::KEY_SIGN {
            self.toggle_sign();
            return;
        }

        if ch.is_ascii_digit() || ch == '.' {
            // Do not build "1.2.3".
            if ch == '.' && self.expression.contains('.') {
                let last_is_operator = self
                    .expression
                    .chars()
                    .last()
                    .map(|c| matches!(c, '+' | '-' | '*' | '/'))
                    .unwrap_or(true);
                if !last_is_operator {
                    return;
                }
            }
        }

        self.expression.push(ch);
        self.evaluate_expression();
    }

    /// Flip the sign of whatever is on screen.
    fn toggle_sign(&mut self) {
        if self.expression.is_empty() {
            self.expression.push('-');
        } else if self.expression.starts_with('-') {
            self.expression.remove(0);
        } else {
            self.expression.insert(0, '-');
        }
        self.evaluate_expression();
    }

    /// Evaluate the current expression, updating the display.
    pub fn evaluate_expression(&mut self) {
        if self.expression.is_empty() {
            self.display = "0".to_string();
            return;
        }
        match parser::evaluate_with(&self.expression, self.angles) {
            Ok(value) => {
                self.display = parser::format_number(value);
            }
            Err(error) => {
                // A half typed expression is not an error yet; only report once
                // there is nothing left to add.
                let complete = self.expression.ends_with(|c: char| c == ')' || c.is_ascii_digit());
                self.display = if complete {
                    format!("Error: {error}")
                } else {
                    String::new()
                };
            }
        }
    }

    /// Evaluate an expression the user submitted, recording it in the history.
    pub fn submit(&mut self, expression: &str) -> String {
        let text = expression.trim();
        if text.is_empty() {
            return self.display.clone();
        }
        self.expression = text.to_string();
        let result = match parser::evaluate_with(text, self.angles) {
            Ok(value) => {
                self.display = parser::format_number(value);
                self.display.clone()
            }
            Err(error) => {
                self.display = format!("Error: {error}");
                self.display.clone()
            }
        };
        self.history.push(HistoryEntry {
            expression: text.to_string(),
            result: result.clone(),
        });
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        result
    }

    /// Evaluate exactly, with no floating point.
    pub fn submit_exact(&mut self, expression: &str) -> String {
        let text = expression.trim();
        if text.is_empty() {
            return "0".to_string();
        }
        let result = match crate::expr::exact::evaluate(text) {
            Ok(value) => value.format(18),
            Err(error) => format!("Error: {error}"),
        };
        self.history.push(HistoryEntry {
            expression: text.to_string(),
            result: result.clone(),
        });
        result
    }

    /// Put a history entry back into the expression buffer.
    pub fn recall(&mut self, index: usize) -> String {
        match self.history.get(index) {
            Some(entry) => {
                self.expression = entry.expression.clone();
                self.display = entry.result.clone();
                entry.expression.clone()
            }
            None => String::new(),
        }
    }

    // --- memory registers -------------------------------------------------

    /// Add the current display to a register.
    pub fn memory_add(&mut self, slot: usize) -> f64 {
        if slot >= self.registers.len() {
            return 0.0;
        }
        let value: f64 = self.display.parse().unwrap_or(0.0);
        self.registers[slot].value += value;
        self.registers[slot].used = true;
        self.registers[slot].value
    }

    /// Subtract the current display from a register.
    pub fn memory_subtract(&mut self, slot: usize) -> f64 {
        if slot >= self.registers.len() {
            return 0.0;
        }
        let value: f64 = self.display.parse().unwrap_or(0.0);
        self.registers[slot].value -= value;
        self.registers[slot].used = true;
        self.registers[slot].value
    }

    /// Recalled register value, or zero.
    pub fn memory_recall(&self, slot: usize) -> f64 {
        self.registers.get(slot).map(|r| r.value).unwrap_or(0.0)
    }

    pub fn memory_clear(&mut self, slot: usize) {
        if let Some(register) = self.registers.get_mut(slot) {
            *register = Register::EMPTY;
        }
    }

    /// Swap the display for a register value.
    pub fn memory_store(&mut self, slot: usize) {
        if slot >= self.registers.len() {
            return;
        }
        let value: f64 = self.display.parse().unwrap_or(0.0);
        self.registers[slot] = Register {
            value,
            used: true,
        };
    }

    /// A snapshot for the UI.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            display: self.display.clone(),
            expression: self.expression.clone(),
            angle_mode: self.angles.short_name(),
            engine: self.engine,
            history: self.history.clone(),
            registers: self.registers,
        }
    }
}

/// Convert a float into the exact representation, for the Big numbers mode.
pub fn to_exact(value: f64) -> Option<Exact> {
    Exact::from_f64(value)
}
