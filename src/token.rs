use core::fmt;

#[derive(Debug, Clone)]
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

// a token consists of a lexeme and its metadata.
#[derive(Debug, Clone)]
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

// add the boring func in order to keep consistent with the test cases of the book <crafting interpreter>.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Type::LeftParen => write!(f, "LEFT_PAREN"),
            Type::RightParen => write!(f, "RIGHT_PAREN"),
            Type::LeftBrace => write!(f, "LEFT_BRACE"),
            Type::RightBrace => write!(f, "RIGHT_BRACE"),
            Type::BangEqual => write!(f, "BANG_EQUAL"),
            Type::EqualEqual => write!(f, "EQUAL_EQUAL"),
            Type::GreaterEqual => write!(f, "GREATER_EQUAL"),
            Type::LessEqual => write!(f, "LESS_EQUAL"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.t {
            Type::STRING { literal } => write!(f, "STRING {} {}", &self.lexeme, literal),
            Type::NUMBER { literal } => write!(f, "NUMBER {} {:?}", &self.lexeme, literal),
            Type::EOF => write!(f, "EOF null"),
            _ => write!(f, "{} {} null", self.t, &self.lexeme),
        }
    }
}
