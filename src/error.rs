use core::fmt;

use crate::token::{Token, TokenType};

fn report(line: u32, pos: &str, msg: &str) {
    eprintln!("[line {}] Error{}: {}", line, pos, msg);
}

pub fn scan_error(line: u32, msg: &str) {
    report(line, "", msg);
}

pub enum Error {
    ParseError(Token, String),
    RuntimeError(Token, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(token, msg) => match token.t {
                TokenType::EOF => Ok(report(token.line, " at end", msg)),
                _ => Ok(report(token.line, &format!(" at '{}'", &token.lexeme), msg)),
            },
            Error::RuntimeError(token, msg) => write!(f, "{}\n[line {}]", msg, token.line),
        }
    }
}
