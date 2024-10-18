use std::{cell::RefCell, rc::Rc};

pub use environment::Environment;
pub use value::Value;

mod environment;
mod value;

use crate::{
    ast::{Expr, Stmt},
    error::Error,
    token::TokenType,
};

// Interpret the semantics of an ast.
pub fn interpret(stmts: Vec<Stmt>) {
    let mut i = Interpreter::new();
    i.interpret(stmts);
}

struct Interpreter {
    env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            env: Rc::new(RefCell::new(Environment::new())),
        }
    }

    fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            if let Some(e) = self.execute(stmt) {
                // runtime error, interpreter will print it.
                eprintln!("{}", e);
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Option<Error> {
        match stmt {
            Stmt::Expression { expr } => match self.eval(expr) {
                Ok(_) => None,
                Err(e) => Some(e),
            },
            Stmt::Print { expr } => match self.eval(expr) {
                Ok(value) => {
                    println!("{}", value);
                    None
                }
                Err(e) => Some(e),
            },
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => match self.eval(expr) {
                        Ok(value) => value,
                        Err(e) => return Some(e),
                    },
                    None => Value::Nil,
                };
                self.env.borrow_mut().define(name.lexeme, value);
                None
            }
            Stmt::Block { stmts } => {
                let previous = self.env.clone();
                let mut res = None;
                self.env = Rc::new(RefCell::new(Environment::from(&self.env)));
                for stmt in stmts {
                    if let Some(e) = self.execute(*stmt) {
                        res = Some(e);
                        break;
                    }
                }
                self.env = previous;
                res
            }
            _ => todo!(),
        }
    }

    fn eval(&mut self, expr: Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal { .. } => self.eval_literal(expr),
            Expr::Grouping { expression } => self.eval(*expression),
            Expr::Unary { .. } => self.eval_unary(expr),
            Expr::Binary { .. } => self.eval_binary(expr),
            Expr::Variable { .. } => self.eval_variable(expr),
            Expr::Assign { .. } => self.eval_assign(expr),
            _ => todo!(),
        }
    }

    fn eval_literal(&mut self, expr: Expr) -> Result<Value, Error> {
        if let Expr::Literal { value } = expr {
            match &value.t {
                TokenType::TRUE => return Ok(Value::Bool(true)),
                TokenType::FALSE => return Ok(Value::Bool(false)),
                TokenType::NIL => return Ok(Value::Nil),
                TokenType::NUMBER { literal } => return Ok(Value::Number(*literal)),
                TokenType::STRING { literal } => return Ok(Value::String(literal.clone())),
                _ => {
                    return Err(Error::RuntimeError(
                        value.clone(),
                        "Expr::Literal error".to_string(),
                    ))
                }
            }
        }
        unreachable!()
    }

    fn eval_unary(&mut self, expr: Expr) -> Result<Value, Error> {
        if let Expr::Unary { op, right } = expr {
            let right = self.eval(*right)?;
            match (&op.t, &right) {
                (TokenType::MINUS, Value::Number(num)) => return Ok(Value::Number(-num)),
                (TokenType::BANG, _) => return Ok(Value::Bool(!right.is_truthy())),
                _ => {
                    return Err(Error::RuntimeError(
                        op.clone(),
                        format!("Expr::Unary's op {} and right {} is mismatch", op, right),
                    ))
                }
            }
        }
        unreachable!()
    }

    fn eval_binary(&mut self, expr: Expr) -> Result<Value, Error> {
        if let Expr::Binary { left, op, right } = expr {
            let left = self.eval(*left)?;
            let right = self.eval(*right)?;
            match &op.t {
                TokenType::MINUS => match left - right {
                    Ok(res) => return Ok(res),
                    Err(e) => return Err(Error::RuntimeError(op.clone(), e)),
                },
                TokenType::PLUS => match left + right {
                    Ok(res) => return Ok(res),
                    Err(e) => return Err(Error::RuntimeError(op.clone(), e)),
                },
                TokenType::SLASH => match left / right {
                    Ok(res) => return Ok(res),
                    Err(e) => return Err(Error::RuntimeError(op.clone(), e)),
                },
                TokenType::STAR => match left * right {
                    Ok(res) => return Ok(res),
                    Err(e) => return Err(Error::RuntimeError(op.clone(), e)),
                },
                TokenType::GREATER => return Ok(Value::Bool(left > right)),
                TokenType::GreaterEqual => return Ok(Value::Bool(left >= right)),
                TokenType::LESS => return Ok(Value::Bool(left < right)),
                TokenType::LessEqual => return Ok(Value::Bool(left <= right)),
                TokenType::BangEqual => return Ok(Value::Bool(left != right)),
                TokenType::EqualEqual => return Ok(Value::Bool(left == right)),
                _ => {
                    return Err(Error::RuntimeError(
                        op.clone(),
                        format!(
                            "Expr:Binary's op {} and left {} or right {} is mismatch",
                            op.clone(),
                            left,
                            right
                        ),
                    ));
                }
            }
        }
        unreachable!()
    }

    fn eval_variable(&self, expr: Expr) -> Result<Value, Error> {
        if let Expr::Variable { name } = expr {
            return self.env.borrow_mut().get(&name);
        }
        unreachable!()
    }

    fn eval_assign(&mut self, expr: Expr) -> Result<Value, Error> {
        if let Expr::Assign { name, value } = expr {
            let value = self.eval(*value)?;
            match self.env.borrow_mut().assign(name, value.clone()) {
                None => return Ok(value),
                Some(e) => return Err(e),
            }
        }
        unreachable!()
    }
}
