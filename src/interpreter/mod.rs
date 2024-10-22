mod environment;
mod value;

use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expr, Stmt},
    error::Error,
    token::TokenType,
};
use environment::Environment;
use value::{Clock, FuncValue, Value};

// Interpret the semantics of an ast.
pub fn interpret(stmts: Vec<Stmt>) {
    let mut i = Interpreter::new();
    i.interpret(stmts);
}

#[derive(Clone)]
struct Interpreter {
    env: Rc<RefCell<Environment>>, // track the current environment: variables, functions, &c.
    globals: Rc<RefCell<Environment>>, // the global environment, e.g.: native functions.
}

impl Interpreter {
    fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        // add native function.
        globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(Clock {})));

        Interpreter {
            env: globals.clone(),
            globals,
        }
    }

    fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in &stmts {
            if let Err(e) = self.execute(stmt) {
                // runtime error, interpreter will print it.
                eprintln!("{}", e);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(Value, bool), Error> {
        match stmt {
            Stmt::Expression { expr } => Ok((self.eval(expr)?, false)),
            Stmt::Print { expr } => {
                let v = self.eval(expr)?;
                println!("{}", v);
                Ok((v, false))
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.eval(expr)?,
                    None => Value::Nil,
                };
                self.env
                    .borrow_mut()
                    .define(name.lexeme.clone(), value.clone());
                Ok((value, false))
            }
            Stmt::Block { stmts } => {
                let previous = self.env.clone();
                let mut res = Ok((Value::Nil, false));
                self.env = Rc::new(RefCell::new(Environment::from(&self.env)));
                for stmt in stmts {
                    let stmt_value = self.execute(&*stmt);
                    match stmt_value {
                        Err(e) => {
                            res = Err(e);
                            break;
                        }
                        Ok(v) => {
                            let val = v.0;
                            if v.1 {
                                res = Ok((val, true));
                                break;
                            } else {
                                res = Ok((val, false));
                            }
                        }
                    }
                }
                self.env = previous;
                res
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.eval(condition)?.is_truthy() {
                    return self.execute(&*then_branch);
                } else if let Some(else_branch) = else_branch {
                    return self.execute(&*else_branch);
                }
                Ok((Value::Nil, false))
            }
            Stmt::While { condition, body } => {
                let mut res = (Value::Nil, false);
                while self.eval(condition)?.is_truthy() {
                    res = self.execute(&*body)?;
                }
                Ok(res)
            }
            Stmt::Return { keyword: _, value } => match value {
                Some(expr) => Ok((self.eval(expr)?, true)),
                None => Ok((Value::Nil, true)),
            },
            Stmt::Function { name, params, body } => {
                let func = FuncValue::from(
                    name.clone(),
                    params.clone(),
                    *body.clone(),
                    self.env.clone(),
                );
                self.env
                    .borrow_mut()
                    .define(name.lexeme.clone(), Value::Callable(Rc::new(func)));

                Ok((Value::Nil, false))
            }
            _ => todo!(),
        }
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal { .. } => self.eval_literal(expr),
            Expr::Grouping { expression } => self.eval(&*expression),
            Expr::Unary { .. } => self.eval_unary(expr),
            Expr::Binary { .. } => self.eval_binary(expr),
            Expr::Variable { .. } => self.eval_variable(expr),
            Expr::Assign { .. } => self.eval_assign(expr),
            Expr::Logical { .. } => self.eval_logical(expr),
            Expr::Call { .. } => self.eval_call(expr),
            _ => todo!(),
        }
    }

    fn eval_literal(&mut self, expr: &Expr) -> Result<Value, Error> {
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

    fn eval_unary(&mut self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Unary { op, right } = expr {
            let right = self.eval(&*right)?;
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

    fn eval_binary(&mut self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Binary { left, op, right } = expr {
            let left = self.eval(&*left)?;
            let right = self.eval(&*right)?;
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
                TokenType::GREATER => return Ok(left.gt(&right)),
                TokenType::GreaterEqual => return Ok(left.ge(&right)),
                TokenType::LESS => return Ok(left.lt(&right)),
                TokenType::LessEqual => return Ok(left.le(&right)),
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

    fn eval_variable(&self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Variable { name } = expr {
            return self.env.borrow_mut().get(&name);
        }
        unreachable!()
    }

    fn eval_assign(&mut self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Assign { name, value } = expr {
            let value = self.eval(&*value)?;
            self.env.borrow_mut().assign(name.clone(), value.clone())?;
            return Ok(value);
        }
        unreachable!()
    }

    // Lox 对 logical or/and 的语义和常见语言不同，比如:
    // - print "hi" or 2;
    //   常见语言: print true
    //   Lox: print "hi"
    // - print nil or "yes";
    //   常见语言: print true
    //   Lox: print "yes"
    fn eval_logical(&mut self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Logical { left, op, right } = expr {
            let left = self.eval(&*left)?;
            if op.t == TokenType::OR {
                if left.is_truthy() {
                    return Ok(left);
                }
            } else {
                // op.t == TokenType::AND
                if !left.is_truthy() {
                    return Ok(left);
                }
            }
            return self.eval(&*right);
        }
        unreachable!()
    }

    fn eval_call(&mut self, expr: &Expr) -> Result<Value, Error> {
        if let Expr::Call {
            callee,
            paren,
            args,
        } = expr
        {
            let callee = match self.eval(&*callee)? {
                Value::Callable(callee) => callee.clone(),
                _ => {
                    return Err(Error::RuntimeError(
                        paren.clone(),
                        "Can only call functions and classes.".to_string(),
                    ))
                }
            };

            if args.len() != callee.arity() {
                return Err(Error::RuntimeError(
                    paren.clone(),
                    format!(
                        "Expected {} arguments but got {}.",
                        callee.arity(),
                        args.len()
                    ),
                ));
            }

            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.eval(arg)?);
            }

            return callee.call(self.clone(), arg_values);
        }
        unreachable!()
    }
}
