//! Expression engine: lexer, parser, numeric tower.

pub mod lexer;
pub mod number;
pub mod parser;

pub use number::{Exact, MathError, MathResult, Number};
pub use parser::{evaluate, evaluate_with, format_number, AngleMode};
