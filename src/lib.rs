pub mod interpreter;
pub mod scanner;
pub mod token;

mod ast;
mod error;
mod parser;

use parser::Parser;

pub fn exec(src: String) {
    let tokens = match scanner::scan_tokens(src) {
        None => return,
        Some(tokens) => tokens,
    };

    let stmts = match Parser::new(tokens).parse() {
        Ok(stmts) => stmts,
        Err(_) => return, // the errors have been print in the `parser::parse()`
    };

    interpreter::interpret(stmts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let src = "print 1 + 100;".to_string();
        exec(src);
    }
}
