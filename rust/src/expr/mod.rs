//! Expression engine: lexer, parser, numeric tower.

pub mod exact;
pub mod lexer;
pub mod number;
pub mod parser;

pub use exact::evaluate as evaluate_exact;
pub use number::{Exact, MathError, MathResult, Number};
pub use parser::{evaluate, evaluate_with, format_number, AngleMode};
