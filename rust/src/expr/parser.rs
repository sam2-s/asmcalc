//! Recursive descent expression parser and evaluator.
//!
//! Grammar, loosest binding first:
//!
//! ```text
//! expression := term (('+' | '-') term)*
//! term       := unary (('*' | '/' | '%') unary)*
//! unary      := ('-' | '+') unary | power
//! power      := postfix ('^' unary)?          // right associative
//! postfix    := primary ('!')?
//! primary    := number | identifier | call | '(' expression ')'
//! call       := identifier '(' args ')'
//! ```
//!
//! `^` is right associative, so `2^3^2` is `2^(3^2)` = 512, not 64. This is the
//! convention every calculator uses and the one people expect.

use super::lexer::{tokenize, Token};
use super::number::{MathError, MathResult, Number};

/// Names the evaluator understands, with their argument counts.
pub struct Function {
    pub name: &'static str,
    pub arity: usize,
}

use MathError::{Domain, Overflow, Undefined};

/// Look up a function by name.
pub fn lookup(name: &str) -> Option<Function> {
    let entry = match name {
        "sin" => ("sin", 1),
        "cos" => ("cos", 1),
        "tan" => ("tan", 1),
        "asin" => ("asin", 1),
        "acos" => ("acos", 1),
        "atan" => ("atan", 1),
        "sinh" => ("sinh", 1),
        "cosh" => ("cosh", 1),
        "tanh" => ("tanh", 1),
        "ln" => ("ln", 1),
        "log" => ("log", 2),
        "log2" => ("log2", 1),
        "log10" => ("log10", 1),
        "exp" => ("exp", 1),
        "sqrt" => ("sqrt", 1),
        "cbrt" => ("cbrt", 1),
        "abs" => ("abs", 1),
        "floor" => ("floor", 1),
        "ceil" => ("ceil", 1),
        "round" => ("round", 1),
        "trunc" => ("trunc", 1),
        "sign" => ("sign", 1),
        "min" => ("min", 2),
        "max" => ("max", 2),
        "pow" => ("pow", 2),
        "hypot" => ("hypot", 2),
        "atan2" => ("atan2", 2),
        "fact" => ("fact", 1),
        "gamma" => ("gamma", 1),
        _ => return None,
    };
    Some(Function {
        name: entry.0,
        arity: entry.1,
    })
}

/// Constants available to expressions.
pub fn constant(name: &str) -> Option<Number> {
    Some(match name {
        "pi" | "PI" => std::f64::consts::PI,
        "e" | "E" => std::f64::consts::E,
        "tau" => std::f64::consts::TAU,
        // The golden ratio has no standard library constant.
        "phi" => 1.618_033_988_749_894_848_2,
        _ => return None,
    })
}

/// How angles are interpreted by the trigonometric functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    Degrees,
    Radians,
    Gradians,
}

impl AngleMode {
    /// Convert a value in this mode to radians.
    pub fn to_radians(self, value: Number) -> Number {
        match self {
            AngleMode::Degrees => value * std::f64::consts::PI / 180.0,
            AngleMode::Radians => value,
            AngleMode::Gradians => value * std::f64::consts::PI / 200.0,
        }
    }

    /// Convert radians back into this mode.
    pub fn from_radians(self, radians: Number) -> Number {
        match self {
            AngleMode::Degrees => radians * 180.0 / std::f64::consts::PI,
            AngleMode::Radians => radians,
            AngleMode::Gradians => radians * 200.0 / std::f64::consts::PI,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            AngleMode::Degrees => "DEG",
            AngleMode::Radians => "RAD",
            AngleMode::Gradians => "GRAD",
        }
    }
}

/// Evaluate an expression, using degrees for trigonometry by default.
pub fn evaluate(input: &str) -> MathResult<Number> {
    evaluate_with(input, AngleMode::Degrees)
}

/// Evaluate an expression with an explicit angle mode.
pub fn evaluate_with(input: &str, angles: AngleMode) -> MathResult<Number> {
    let tokens = tokenize(input).map_err(|e| MathError::Undefined(e.message))?;
    if tokens.is_empty() {
        return Err(MathError::Undefined("empty expression".into()));
    }
    let mut parser = Parser {
        tokens,
        position: 0,
        angles,
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
    angles: AngleMode,
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

    fn expression(&mut self) -> MathResult<Number> {
        let mut value = self.term()?;
        loop {
            if self.eat(&Token::Plus) {
                let right = self.term()?;
                value += right;
            } else if self.eat(&Token::Minus) {
                let right = self.term()?;
                value -= right;
            } else {
                return Ok(value);
            }
        }
    }

    fn term(&mut self) -> MathResult<Number> {
        let mut value = self.unary()?;
        loop {
            if self.eat(&Token::Star) {
                let right = self.unary()?;
                value *= right;
            } else if self.eat(&Token::Slash) {
                let right = self.unary()?;
                // In floating point, dividing by zero yields infinity rather
                // than trapping, so it has to be caught here or `1/0` would
                // quietly display `inf` instead of an error.
                if right == 0.0 {
                    return Err(MathError::DivisionByZero);
                }
                value = value / right;
            } else if self.eat(&Token::Percent) {
                // `x % y` is the remainder, matching the pocket calculator pad.
                let right = self.unary()?;
                if right == 0.0 {
                    return Err(MathError::DivisionByZero);
                }
                value %= right;
            } else {
                return Ok(value);
            }
        }
    }

    fn unary(&mut self) -> MathResult<Number> {
        if self.eat(&Token::Minus) {
            return Ok(-self.unary()?);
        }
        if self.eat(&Token::Plus) {
            return self.unary();
        }
        self.power()
    }

    fn power(&mut self) -> MathResult<Number> {
        let base = self.postfix()?;
        if self.eat(&Token::Caret) {
            // Right associative: the exponent is a full unary expression, so
            // `2^3^2` parses as `2^(3^2)`.
            let exponent = self.unary()?;
            let result = base.powf(exponent);
            if !result.is_finite() {
                return Err(Overflow);
            }
            return Ok(result);
        }
        Ok(base)
    }

    fn postfix(&mut self) -> MathResult<Number> {
        let mut value = self.primary()?;
        while self.eat(&Token::Bang) {
            if value < 0.0 || value.fract() != 0.0 {
                return Err(Domain("factorial of a non-integer".into()));
            }
            if value > 170.0 {
                // Past this the double overflows to infinity.
                return Err(Overflow);
            }
            let mut product = 1.0;
            let mut i = 2;
            while i <= value as u64 {
                product *= i as Number;
                i += 1;
            }
            value = product;
        }
        Ok(value)
    }

    fn primary(&mut self) -> MathResult<Number> {
        let token = self
            .next()
            .ok_or_else(|| MathError::Undefined("expression ended early".into()))?;

        match token {
            Token::Number(value) => Ok(value),
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
                } else if let Some(value) = constant(&name) {
                    Ok(value)
                } else if let Some(function) = lookup(&name) {
                    // A function used without parentheses and without arguments.
                    if function.arity == 1 {
                        Err(MathError::Undefined(format!("`{name}` needs an argument")))
                    } else {
                        Err(Undefined(format!("`{name}` needs arguments")))
                    }
                } else {
                    Err(Undefined(name))
                }
            }
            other => Err(Undefined(format!("unexpected {}", other.describe()))),
        }
    }

    fn call(&mut self, name: &str) -> MathResult<Number> {
        let function = lookup(name).ok_or_else(|| Undefined(name.to_string()))?;

        let mut args = Vec::with_capacity(function.arity);
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

        if args.len() != function.arity {
            return Err(Undefined(format!(
                "`{name}` takes {} argument(s), got {}",
                function.arity,
                args.len()
            )));
        }

        self.apply(function.name, &args)
    }

    fn apply(&self, name: &str, args: &[Number]) -> MathResult<Number> {
        let radians = |x: Number| self.angles.to_radians(x);

        let value = match (name, args) {
            ("sin", [x]) => libm::sin(radians(*x)),
            ("cos", [x]) => libm::cos(radians(*x)),
            ("tan", [x]) => libm::tan(radians(*x)),
            ("asin", [x]) => {
                if !(-1.0..=1.0).contains(x) {
                    return Err(Domain("asin".into()));
                }
                self.angles.from_radians(libm::asin(*x))
            }
            ("acos", [x]) => {
                if !(-1.0..=1.0).contains(x) {
                    return Err(Domain("acos".into()));
                }
                self.angles.from_radians(libm::acos(*x))
            }
            ("atan", [x]) => self.angles.from_radians(libm::atan(*x)),
            ("atan2", [y, x]) => self.angles.from_radians(libm::atan2(*y, *x)),
            ("sinh", [x]) => libm::sinh(*x),
            ("cosh", [x]) => libm::cosh(*x),
            ("tanh", [x]) => libm::tanh(*x),
            ("ln", [x]) => {
                if *x <= 0.0 {
                    return Err(Domain("ln".into()));
                }
                libm::log(*x)
            }
            ("log", [x, base]) => {
                if *x <= 0.0 || *base <= 0.0 || *base == 1.0 {
                    return Err(Domain("log".into()));
                }
                libm::log(*x) / libm::log(*base)
            }
            ("log2", [x]) => {
                if *x <= 0.0 {
                    return Err(Domain("log2".into()));
                }
                libm::log2(*x)
            }
            ("log10", [x]) => {
                if *x <= 0.0 {
                    return Err(Domain("log10".into()));
                }
                libm::log10(*x)
            }
            ("exp", [x]) => libm::exp(*x),
            ("sqrt", [x]) => {
                if *x < 0.0 {
                    return Err(Domain("sqrt".into()));
                }
                libm::sqrt(*x)
            }
            ("cbrt", [x]) => libm::cbrt(*x),
            ("abs", [x]) => libm::fabs(*x),
            ("floor", [x]) => libm::floor(*x),
            ("ceil", [x]) => libm::ceil(*x),
            ("round", [x]) => libm::round(*x),
            ("trunc", [x]) => libm::trunc(*x),
            ("sign", [x]) => {
                if *x > 0.0 {
                    1.0
                } else if *x < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }
            ("min", [a, b]) => a.min(*b),
            ("max", [a, b]) => a.max(*b),
            ("pow", [a, b]) => a.powf(*b),
            ("hypot", [a, b]) => libm::hypot(*a, *b),
            ("fact", [x]) => {
                if *x < 0.0 || x.fract() != 0.0 {
                    return Err(Domain("factorial of a non-integer".into()));
                }
                if *x > 170.0 {
                    return Err(Overflow);
                }
                let mut product = 1.0;
                let mut i = 2;
                while i <= *x as u64 {
                    product *= i as Number;
                    i += 1;
                }
                product
            }
            ("gamma", [x]) => libm::tgamma(*x),
            _ => return Err(Undefined(name.to_string())),
        };

        if !value.is_finite() {
            return Err(Overflow);
        }
        Ok(value)
    }
}

/// Render a float for a calculator display: no trailing zeros, no `1.0`.
pub fn format_number(value: Number) -> String {
    if value == value.trunc() && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let mut text = format!("{value:.10}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}
