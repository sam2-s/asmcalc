//! Tokenizer for the expression engine.
//!
//! Turns a user string into a flat token list. The parser consumes these; the
//! lexer never evaluates anything, so a malformed expression fails here rather
//! than half way through a calculation.

/// One token from an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    /// `!` postfix, as in `5!`
    Bang,
}

impl Token {
    /// How this token reads in an error message.
    pub fn describe(&self) -> String {
        match self {
            Token::Number(n) => format!("number {n}"),
            Token::Identifier(name) => format!("name `{name}`"),
            Token::Plus => "`+`".into(),
            Token::Minus => "`-`".into(),
            Token::Star => "`*`".into(),
            Token::Slash => "`/`".into(),
            Token::Percent => "`%`".into(),
            Token::Caret => "`^`".into(),
            Token::LParen => "`(`".into(),
            Token::RParen => "`)`".into(),
            Token::Comma => "`,`".into(),
            Token::Bang => "`!`".into(),
        }
    }
}

/// A failure while reading an expression.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    /// Character offset the error was found at.
    pub position: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at position {}", self.message, self.position)
    }
}

impl std::error::Error for LexError {}

fn error(message: impl Into<String>, position: usize) -> LexError {
    LexError {
        message: message.into(),
        position,
    }
}

/// Split an expression into tokens.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            let start = i;
            let mut seen_dot = false;
            while i < chars.len() {
                let d = chars[i];
                if d.is_ascii_digit() {
                    i += 1;
                } else if d == '.' && !seen_dot {
                    seen_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let value = text
                .parse::<f64>()
                .map_err(|_| error(format!("`{text}` is not a number"), start))?;
            tokens.push(Token::Number(value));
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            tokens.push(Token::Identifier(name));
            continue;
        }

        let token = match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '^' => Token::Caret,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            '!' => Token::Bang,
            other => return Err(error(format!("unexpected character `{other}`"), i)),
        };
        tokens.push(token);
        i += 1;
    }

    Ok(tokens)
}
