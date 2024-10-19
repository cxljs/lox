use crate::{
    ast::{Expr, Stmt},
    error::Error,
    token::{self, Token, TokenType},
};

// the Lox language defines the syntactic grammar (https://craftinginterpreters.com/appendix-i.html).
// the Parser uses the syntactic grammar to parse the linear sequence of tokens into a nested syntax tree.
// the parser uses recursive descent algo.
// Each method of parser parses a syntactic grammar rule and produces a syntax tree for that rule to the caller.
pub struct Parser {
    tokens: Vec<Token>,
    cur: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, cur: 0 }
    }

    // program -> declaration* EOF ;
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<Error>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();
        while !self.end() {
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    // parse error, compiler/interpreter will print it.
                    eprintln!("{}", e);
                    errors.push(e);
                    self.synchronize();
                }
            }
        }
        match errors.len() {
            0 => Ok(stmts),
            _ => Err(errors),
        }
    }

    // declaration -> classDecl | funDecl | varDecl | statement ;
    // Lox 定义有些地方不能是 classDecl / funDecl / varDecl, 其他 stmt 都可以, 所以把这3个从 stmt 中提出来.
    fn declaration(&mut self) -> Result<Stmt, Error> {
        let token = self.peek();
        match &token.t {
            TokenType::VAR => self.var_decl(),
            TokenType::CLASS => todo!(),
            TokenType::FUN => todo!(),
            _ => self.statement(),
        }
    }

    // varDecl -> "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_decl(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::VAR, "Expect keyword VAR")?;

        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name")?;
        let initializer = match self.r#match(&[TokenType::EQUAL]) {
            true => Some(self.expression()?),
            false => None,
        };
        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    // statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
    fn statement(&mut self) -> Result<Stmt, Error> {
        match self.peek().t {
            TokenType::PRINT => self.print_stmt(),
            TokenType::LeftBrace => self.block(),
            TokenType::IF => self.if_stmt(),
            _ => self.expr_stmt(),
        }
    }

    // printStmt -> "print" expression ";" ;
    fn print_stmt(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::PRINT, "Expect keyword 'PRINT'.")?;
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print { expr: value })
    }

    // block -> "{" declaration* "}" ;
    fn block(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftBrace, "Expect '{'.")?;

        let mut stmts = Vec::new();

        while !self.end() && !self.check(&TokenType::RightBrace) {
            stmts.push(Box::new(self.declaration()?));
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block { stmts })
    }

    // ifStmt -> "if" "(" expression ")" statement ( "else" statement )? ;
    fn if_stmt(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::IF, "Expect keyword 'if'.")?;
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = self.statement()?;
        let else_branch = match self.r#match(&[TokenType::ELSE]) {
            true => Some(Box::new(self.statement()?)),
            false => None,
        };
        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    // exprStmt -> expression ";" ;
    fn expr_stmt(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expr })
    }

    // expression -> assignment ;
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    // assignment -> ( call "." )? IDENTIFIER "=" assignment | logic_or ;
    // logic_or -> logic_and ( "or" logic_and )* ;
    // logic_and -> equality ( "and" equality )* ;
    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;

        if self.r#match(&[TokenType::EQUAL]) {
            let equal = self.previous();
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                _ => {
                    return Err(Error::ParseError(
                        equal,
                        "Invalid assignment target.".to_string(),
                    ))
                }
            }
        }

        Ok(expr)
    }

    // equality -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.r#match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.r#match(&[
            TokenType::GREATER,
            TokenType::GreaterEqual,
            TokenType::LESS,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // term -> factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.r#match(&[TokenType::MINUS, TokenType::PLUS]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // factor -> unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.r#match(&[TokenType::SLASH, TokenType::STAR]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // unary -> ( "!" | "-" ) unary | call ;
    // TODO(cxl): call -> primary
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.r#match(&[TokenType::BANG, TokenType::MINUS]) {
            let op = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    // primary -> "true" | "false" | "nil" | "this" | NUMBER | STRING | IDENTIFIER | "(" expression ")"
    //            | "super" "." IDENTIFIER ;
    fn primary(&mut self) -> Result<Expr, Error> {
        for t in [
            TokenType::TRUE,
            TokenType::FALSE,
            TokenType::NIL,
            TokenType::NUMBER { literal: 0.0 },
            TokenType::STRING {
                literal: "".to_string(),
            },
        ] {
            if self.r#match(&[t]) {
                return Ok(Expr::Literal {
                    value: self.previous(),
                });
            }
        }

        if self.r#match(&[TokenType::IDENTIFIER]) {
            return Ok(Expr::Variable {
                name: self.previous(),
            });
        }

        if self.r#match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        panic!("this | super | id is unimplemented.");
    }

    fn consume(&mut self, t: TokenType, msg: &str) -> Result<Token, Error> {
        if self.check(&t) {
            return Ok(self.advance());
        }
        Err(Error::ParseError(self.peek().clone(), msg.to_owned()))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.end() {
            if self.previous().t == TokenType::SEMICOLON {
                return;
            }
            match &self.peek().t {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,
                _ => (),
            }
            self.advance();
        }
    }

    fn r#match(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t: &token::TokenType) -> bool {
        if self.end() {
            return false;
        }
        let peek = self.peek().clone().t;

        match peek {
            TokenType::NUMBER { .. } => match t {
                TokenType::NUMBER { .. } => return true,
                _ => return false,
            },
            TokenType::STRING { .. } => match t {
                TokenType::STRING { .. } => return true,
                _ => return false,
            },
            _ => (),
        }
        self.peek().t == *t
    }

    fn advance(&mut self) -> Token {
        if !self.end() {
            self.cur += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.cur]
    }

    fn end(&self) -> bool {
        self.peek().t == TokenType::EOF
    }

    fn previous(&self) -> Token {
        self.tokens[self.cur - 1].clone()
    }
}
