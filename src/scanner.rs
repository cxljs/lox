use crate::error;
use crate::token::{self, Token, TokenType, F64};

// the Lox language defines the lexical grammar:
//   NUMBER -> DIGIT+ ("." DIGIT+)? ;
//   STRING -> "\"" <any char except "\"">* "\"" ;
//   IDENTIFIER -> ALPHA (ALPHA | DIGIT)* ;
//   ALPHA -> "a"..."z" |  "A"..."Z" | "_" ;
//   DIGIT -> "0"..."9" ;
// Scanner uses the lexical grammar to transform the source code into tokens.
struct Scanner {
    src: String,
    start: usize,
    cur: usize,
    line: u32,
    tokens: Vec<Token>,
    has_err: bool,
}

pub fn scan_tokens(src: String) -> Option<Vec<Token>> {
    let mut scanner = Scanner::new(src);
    scanner.scan();
    if scanner.has_err {
        return None;
    }
    Some(scanner.tokens)
}

impl Scanner {
    fn new(src: String) -> Scanner {
        Scanner {
            src,
            start: 0,
            cur: 0,
            line: 1,
            tokens: Vec::new(),
            has_err: false,
        }
    }

    fn scan(&mut self) -> &Vec<Token> {
        while !self.end() {
            self.start = self.cur;
            self.scan_next();
        }
        self.tokens
            .push(Token::new(token::TokenType::EOF, "", self.line));

        &self.tokens
    }

    fn scan_next(&mut self) {
        let c = self.advance();
        match c as char {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),
            '!' => match self.r#match('=') {
                true => self.add_token(TokenType::BangEqual),
                false => self.add_token(TokenType::BANG),
            },
            '=' => match self.r#match('=') {
                true => self.add_token(TokenType::EqualEqual),
                false => self.add_token(TokenType::EQUAL),
            },
            '<' => match self.r#match('=') {
                true => self.add_token(TokenType::LessEqual),
                false => self.add_token(TokenType::LESS),
            },
            '>' => match self.r#match('=') {
                true => self.add_token(TokenType::GreaterEqual),
                false => self.add_token(TokenType::GREATER),
            },
            '/' => match self.r#match('/') {
                true => {
                    // comments are lexemes, but they aren't meaningful, and the parser doesn't want to deal
                    // with them, so we don't call add_token()
                    while !self.end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                false => self.add_token(TokenType::SLASH),
            },
            ' ' | '\r' | '\t' => (), // ignore whitespace
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),

            _ => {
                if is_digit(c) {
                    // edge case: negative number:
                    // -123 is not a number literal, instead is an expression.
                    self.number();
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    error::scan_error(self.line, format!("Unexpected character: {}", c).as_str());
                    self.has_err = true;
                }
            }
        }
    }

    fn end(&self) -> bool {
        self.cur >= self.src.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.src.bytes().nth(self.cur).unwrap();
        self.cur += 1;
        ch as char
    }

    fn peek(&self) -> char {
        if self.end() {
            return '\0';
        }
        self.src.bytes().nth(self.cur).unwrap() as char
    }

    fn peek_next(&self) -> char {
        if self.cur + 1 >= self.src.len() {
            return '\0';
        }
        self.src.bytes().nth(self.cur + 1).unwrap() as char
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.end() {
            return false;
        }
        if self.src.bytes().nth(self.cur).unwrap() as char != expected {
            return false;
        }
        self.cur += 1;
        true
    }

    fn string(&mut self) {
        while !self.end() && self.peek() != '"' {
            // Lox supports multi-line string.
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.end() {
            error::scan_error(self.line, "Unterminated string.");
            return;
        }

        // the closing ".
        self.advance();

        // trim the surrounding quotes.
        let literal = self.src.get(self.start + 1..self.cur - 1).unwrap();
        self.add_token(TokenType::STRING {
            literal: literal.to_string(),
        });
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        // look for a fractional part.
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let num = self
            .src
            .get(self.start..self.cur)
            .unwrap()
            .parse::<f64>()
            .unwrap();
        self.add_token(TokenType::NUMBER { literal: F64(num) });
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let lexeme = self.src.get(self.start..self.cur).unwrap();
        self.add_token(TokenType::keyword_or_id(lexeme));
    }

    fn add_token(&mut self, t: TokenType) {
        let lexeme = self.src.get(self.start..self.cur).unwrap();
        self.tokens.push(Token::new(t, lexeme, self.line));
    }
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}
