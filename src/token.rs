use core::fmt;

#[derive(Debug)]
pub enum Type {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BangEqual,
    EQUAL,
    EqualEqual,
    GREATER,
    GreaterEqual,
    LESS,
    LessEqual,

    // Literals.
    IDENTIFIER,
    STRING { literal: String },
    NUMBER { literal: f64 }, // all numbers in Lox are floating point at runtime.

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

impl Type {
    pub fn keyword_or_id(s: &str) -> Type {
        match s {
            "and" => Type::AND,
            "class" => Type::CLASS,
            "else" => Type::ELSE,
            "false" => Type::FALSE,
            "for" => Type::FOR,
            "fun" => Type::FUN,
            "if" => Type::IF,
            "nil" => Type::NIL,
            "or" => Type::OR,
            "print" => Type::PRINT,
            "return" => Type::RETURN,
            "super" => Type::SUPER,
            "this" => Type::THIS,
            "true" => Type::TRUE,
            "var" => Type::VAR,
            "while" => Type::WHILE,
            _ => Type::IDENTIFIER,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    t: Type,
    lexeme: String,
    line: u32, // location info
}

impl Token {
    pub fn new(t: Type, lexeme: &str, line: u32) -> Token {
        Token {
            t,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.t {
            Type::STRING { literal } => write!(f, "String \"{:?}\" {:?}", self.lexeme, literal),
            Type::NUMBER { literal } => write!(f, "Number {:?} {:?}", self.lexeme, literal),
            Type::EOF => write!(f, "EOF null"),
            _ => write!(f, "{:?} {:} null", self.t, self.lexeme),
        }
    }
}
