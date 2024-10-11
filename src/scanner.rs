use crate::error;
use crate::token::{self, Token, Type};

struct Scanner {
    src: String,
    start: usize,
    cur: usize,
    line: u32,
    tokens: Vec<Token>,
}

pub fn scan_tokens(src: String) -> Vec<Token> {
    let mut scanner = Scanner::new(src);
    scanner.scan();
    scanner.tokens
}

impl Scanner {
    fn new(src: String) -> Scanner {
        Scanner {
            src,
            start: 0,
            cur: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    fn scan(&mut self) -> &Vec<Token> {
        while !self.end() {
            self.start = self.cur;
            self.scan_next();
        }
        self.tokens
            .push(Token::new(token::Type::EOF, "", self.line));

        &self.tokens
    }

    fn scan_next(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(Type::LeftParen),
            ')' => self.add_token(Type::RightParen),
            '{' => self.add_token(Type::LeftBrace),
            '}' => self.add_token(Type::RightBrace),
            ',' => self.add_token(Type::COMMA),
            '.' => self.add_token(Type::DOT),
            '-' => self.add_token(Type::MINUS),
            '+' => self.add_token(Type::PLUS),
            ';' => self.add_token(Type::SEMICOLON),
            '*' => self.add_token(Type::STAR),
            '!' => match self.r#match('=') {
                true => self.add_token(Type::BangEqual),
                false => self.add_token(Type::BANG),
            },
            '=' => match self.r#match('=') {
                true => self.add_token(Type::EqualEqual),
                false => self.add_token(Type::EQUAL),
            },
            '<' => match self.r#match('=') {
                true => self.add_token(Type::LessEqual),
                false => self.add_token(Type::LESS),
            },
            '>' => match self.r#match('=') {
                true => self.add_token(Type::GreaterEqual),
                false => self.add_token(Type::GREATER),
            },
            '/' => match self.r#match('/') {
                true => {
                    // comments are lexemes, but they aren't meaningful, and the parser doesn't want to deal
                    // with them, so we don't call add_token()
                    while !self.end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                false => self.add_token(Type::SLASH),
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
                    error::error(self.line, "unexpected character.");
                }
            }
        }
    }

    fn end(&self) -> bool {
        self.cur >= self.src.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.src.chars().nth(self.cur).unwrap();
        self.cur += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.end() {
            return '\0';
        }
        self.src.chars().nth(self.cur).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.cur + 1 >= self.src.len() {
            return '\0';
        }
        self.src.chars().nth(self.cur + 1).unwrap()
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.end() {
            return false;
        }
        if self.src.chars().nth(self.cur).unwrap() != expected {
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
            error::error(self.line, "unterminated string");
            return;
        }

        // the closing ".
        self.advance();

        // trim the surrounding quotes.
        let literal = self.src.get(self.start + 1..self.cur - 1).unwrap();
        self.add_token(Type::STRING {
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
        self.add_token(Type::NUMBER { literal: num });
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let lexeme = self.src.get(self.start..self.cur).unwrap();
        self.add_token(Type::keyword_or_id(lexeme));
    }

    fn add_token(&mut self, t: Type) {
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
