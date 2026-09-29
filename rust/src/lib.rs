//! Rust feature layer for the AArch64 assembly calculator.
//!
//! The arithmetic kernel is assembly and lives in `app/src/main/asm`. Rust owns
//! everything layered on top of it: expression parsing, transcendentals, unit
//! conversion, number bases, arbitrary precision, matrices and complex numbers.

pub mod expr;
pub mod fixed;
pub mod kernel;
