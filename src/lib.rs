pub mod interpreter;
pub mod scanner;
pub mod token;

mod ast;
mod error;
mod parser;
mod resolver;

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

    let locations = match resolver::resolve_variable(&stmts) {
        Some(locations) => locations,
        None => return,
    };

    interpreter::interpret(stmts, locations);
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
