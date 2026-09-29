//! Exact evaluation over the decimal tower.
//!
//! This is a second parser, separate from the `f64` one in `parser.rs`, because
//! the two cannot be merged: `sqrt(2)` has no exact decimal answer, and
//! `1/3` has no exact `f64` answer. Rather than approximate in both directions,
//! each engine answers what it can answer exactly and reports the rest.
//!
//! Supported: `+ - * /`, parentheses, unary minus, `^`, `!`, and the functions
//! that stay inside the rationals (`abs`, `min`, `max`, `gcd`, `mod`).

use super::lexer::{tokenize, Token};
use super::number::{factorial, Exact, MathError, MathResult};

/// Evaluate an expression exactly.
pub fn evaluate(input: &str) -> MathResult<Exact> {
    let tokens = tokenize(input).map_err(|e| MathError::Undefined(e.message))?;
    if tokens.is_empty() {
        return Err(MathError::Undefined("empty expression".into()));
    }
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let value = parser.expression()?;
    if parser.position != parser.tokens.len() {
        let token = parser.tokens[parser.position].describe();
        return Err(MathError::Undefined(format!("unexpected {token}")));
    }
    Ok(value)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.peek() == Some(token) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expression(&mut self) -> MathResult<Exact> {
        let mut value = self.term()?;
        loop {
            if self.eat(&Token::Plus) {
                let right = self.term()?;
                value = value.add(&right);
            } else if self.eat(&Token::Minus) {
                let right = self.term()?;
                value = value.sub(&right);
            } else {
                return Ok(value);
            }
        }
    }

    fn term(&mut self) -> MathResult<Exact> {
        let mut value = self.unary()?;
        loop {
            if self.eat(&Token::Star) {
                let right = self.unary()?;
                value = value.mul(&right);
            } else if self.eat(&Token::Slash) {
                let right = self.unary()?;
                value = value
                    .div(&right)
                    .ok_or(MathError::DivisionByZero)?;
            } else if self.eat(&Token::Percent) {
                let right = self.unary()?;
                value = value.rem(&right).ok_or(MathError::DivisionByZero)?;
            } else {
                return Ok(value);
            }
        }
    }

    fn unary(&mut self) -> MathResult<Exact> {
        if self.eat(&Token::Minus) {
            return Ok(self.unary()?.negate());
        }
        if self.eat(&Token::Plus) {
            return self.unary();
        }
        self.power()
    }

    fn power(&mut self) -> MathResult<Exact> {
        let base = self.postfix()?;
        if self.eat(&Token::Caret) {
            let exponent = self.unary()?;
            let n = exponent
                .to_i64()
                .ok_or_else(|| MathError::Domain("exponent is not a whole number".into()))?;
            return base
                .powi(n)
                .ok_or_else(|| MathError::Domain("this power".into()));
        }
        Ok(base)
    }

    fn postfix(&mut self) -> MathResult<Exact> {
        let mut value = self.primary()?;
        while self.eat(&Token::Bang) {
            let n = value
                .to_i64()
                .ok_or_else(|| MathError::Domain("factorial of a non-integer".into()))?;
            if n < 0 {
                return Err(MathError::Domain("factorial of a negative number".into()));
            }
            value = factorial(n as u64).ok_or(MathError::Overflow)?;
        }
        Ok(value)
    }

    fn primary(&mut self) -> MathResult<Exact> {
        let token = self
            .next()
            .ok_or_else(|| MathError::Undefined("expression ended early".into()))?;

        match token {
            Token::Number(value) => {
                Exact::from_f64(value).ok_or(MathError::Overflow)
            }
            Token::LParen => {
                let value = self.expression()?;
                if !self.eat(&Token::RParen) {
                    return Err(MathError::Undefined("missing `)`".into()));
                }
                Ok(value)
            }
            Token::Identifier(name) => {
                if self.eat(&Token::LParen) {
                    self.call(&name)
                } else {
                    Err(MathError::Undefined(name))
                }
            }
            other => Err(MathError::Undefined(format!(
                "unexpected {} in exact mode",
                other.describe()
            ))),
        }
    }

    fn call(&mut self, name: &str) -> MathResult<Exact> {
        let mut args = Vec::new();
        if !self.eat(&Token::RParen) {
            loop {
                args.push(self.expression()?);
                if self.eat(&Token::Comma) {
                    continue;
                }
                if self.eat(&Token::RParen) {
                    break;
                }
                return Err(MathError::Undefined("missing `)`".into()));
            }
        }

        match (name, args.as_slice()) {
            ("abs", [x]) => Ok(x.clone().abs()),
            ("min", [a, b]) => Ok(if a <= b { a.clone() } else { b.clone() }),
            ("max", [a, b]) => Ok(if a >= b { a.clone() } else { b.clone() }),
            ("mod", [a, b]) => a.rem(b).ok_or(MathError::DivisionByZero),
            ("pow", [a, b]) => {
                let n = b
                    .to_i64()
                    .ok_or_else(|| MathError::Domain("exponent is not a whole number".into()))?;
                a.powi(n).ok_or_else(|| MathError::Domain("this power".into()))
            }
            // pi and e are not exact decimals, so exact mode refuses them
            // rather than inventing a value.
            _ => Err(MathError::Domain(format!(
                "`{name}` has no exact decimal answer"
            ))),
        }
    }
}
