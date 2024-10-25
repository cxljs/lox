use core::fmt;

// Rust f64 不能直接实现 trait Eq & Hash, 所以封装一下.
#[derive(Debug, Clone)]
pub struct F64(pub f64);

impl std::cmp::PartialEq for F64 {
    fn eq(&self, oth: &Self) -> bool {
        self.0 == oth.0
    }
}

impl std::cmp::Eq for F64 {}

impl std::hash::Hash for F64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as u64).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,  // (
    RightParen, // )
    LeftBrace,  // {
    RightBrace, // }
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
    // bool, nil 也能当字面量处理，这里把它们看成关键字
    IDENTIFIER,
    STRING { literal: String },
    NUMBER { literal: F64 }, // all numbers in Lox are floating point at runtime.

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

impl TokenType {
    pub fn keyword_or_id(s: &str) -> TokenType {
        match s {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "for" => TokenType::FOR,
            "fun" => TokenType::FUN,
            "if" => TokenType::IF,
            "nil" => TokenType::NIL,
            "or" => TokenType::OR,
            "print" => TokenType::PRINT,
            "return" => TokenType::RETURN,
            "super" => TokenType::SUPER,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "var" => TokenType::VAR,
            "while" => TokenType::WHILE,
            _ => TokenType::IDENTIFIER,
        }
    }
}

// a token consists of a lexeme and its metadata.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Token {
    pub t: TokenType,
    pub lexeme: String,
    pub line: u32, // location info
}

impl Token {
    pub fn new(t: TokenType, lexeme: &str, line: u32) -> Token {
        Token {
            t,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}

// add the boring func in order to keep consistent with the test cases of the book <crafting interpreter>.
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TokenType::LeftParen => write!(f, "LEFT_PAREN"),
            TokenType::RightParen => write!(f, "RIGHT_PAREN"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE"),
            TokenType::BangEqual => write!(f, "BANG_EQUAL"),
            TokenType::EqualEqual => write!(f, "EQUAL_EQUAL"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL"),
            TokenType::LessEqual => write!(f, "LESS_EQUAL"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.t {
            TokenType::STRING { literal } => write!(f, "STRING {} {}", &self.lexeme, literal),
            TokenType::NUMBER { literal } => write!(f, "NUMBER {} {:?}", &self.lexeme, literal),
            TokenType::EOF => write!(f, "EOF null"),
            _ => write!(f, "{} {} null", self.t, &self.lexeme),
        }
    }
}
