// The Lox language defines the syntactic grammar:
// (doc: https://craftinginterpreters.com/appendix-i.html)
// a Lox program is a series of declarations:
//   program -> declaration* EOF ;
//   declaration -> classDecl | funDecl | varDecl | statement ;
//
//   classDecl -> "class" IDENTIFIER ( "<" IDENTIFIER )? "{" function* "}" ;
//   funDecl -> "fun" function ;
//   varDecl -> "var" IDENTIFIER ( "=" expression )? ";" ;
//   statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
//
//   exprStmt -> expression ";" ;
//   forStmt -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
//   ifStmt -> "if" "(" expression ")" statement ( "else" statement )? ;
//   printStmt -> "print" expression ";" ;
//   returnStmt -> "return" expression? ";" ;
//   whileStmt -> "while" "(" expression ")" statement ;
//   block -> "{" declaration* "}" ;
//
// expression produce values.
// Lox uses a separate rule for each precedence level to make it explicit.
//   expression -> assignment ;
//   assignment -> ( call "." )? IDENTIFIER "=" assignment | logic_or ;
//
//   logic_or -> logic_and ( "or" logic_and )* ;
//   logic_and -> equality ( "and" equality )* ;
//   equality -> comparison ( ( "!=" | "==" ) comparison )* ;
//   comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//   term -> factor ( ( "-" | "+" ) factor )* ;
//   factor -> unary ( ( "/" | "*" ) unary )* ;
//
//   unary -> ( "!" | "-" ) unary | call ;
//   call -> primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
//   primary -> "true" | "false" | "nil" | "this" | NUMBER | STRING | IDENTIFIER | "(" expression ")" | "super" "." IDENTIFIER ;
//
// helper rules:
//   function -> IDENTIFIER "(" parameters? ")" block ;
//   parameters -> IDENTIFIER ( "," IDENTIFIER )* ;
//   arguments -> expression ( "," expression )* ;
//

use crate::token::Token;

// Expr = a list of tokens
pub enum Expr {
    // literal value
    Literal {
        value: Token,
    },
    // unary operators
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    // binary operator
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    // using parentheses to group expressions
    Grouping {
        expression: Box<Expr>,
    },
    // variable assignment
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    // function call
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Box<Expr>>,
    },
    // logical and/or
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    // variable access expressions
    Variable {
        name: Token,
    },
    // class property access
    Get {
        object: Box<Expr>,
        name: Token,
    },
    // class property assignment
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    // super expression in inheritance
    Super {
        keyword: Token,
        method: Token,
    },
    // this expression
    This {
        keyword: Token,
    },
}

// Stmt = a list of Exprs(and Tokens)
pub enum Stmt {
    Block {
        stmts: Vec<Box<Stmt>>,
    },
    Class {
        name: Token,
        super_class: Expr,       // Expr::Variable
        methods: Vec<Box<Stmt>>, // Stmt::Function
    },
    Expression {
        expr: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expr: Expr,
    },
    Return {
        keyword: Token,
        value: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
