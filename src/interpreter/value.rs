use core::fmt;
use std::{
    cell::RefCell,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{environment::Environment, Interpreter};
use crate::{
    ast::Stmt,
    error::Error,
    token::{Token, TokenType},
};

pub trait Callable {
    fn call(&self, i: Interpreter, args: Vec<Value>) -> Result<Value, Error>;
    fn arity(&self) -> usize; // return the number of arguments of function or operation expects.
    fn to_string(&self) -> String;
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
}

impl Value {
    // Lox follows Ruby's simple rule: false and nil are false, and everything else is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }
    /*
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }
    */
    pub fn ge(&self, oth: &Self) -> Value {
        match (self, oth) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
            _ => Value::Bool(false),
        }
    }

    pub fn gt(&self, oth: &Self) -> Value {
        match (self, oth) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
            _ => Value::Bool(false),
        }
    }
    pub fn le(&self, oth: &Self) -> Value {
        match (self, oth) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
            _ => Value::Bool(false),
        }
    }
    pub fn lt(&self, oth: &Self) -> Value {
        match (self, oth) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
            _ => Value::Bool(false),
        }
    }
}

impl TryFrom<&TokenType> for Value {
    type Error = String;
    fn try_from(t: &TokenType) -> Result<Value, Self::Error> {
        match t {
            TokenType::NIL => Ok(Value::Nil),
            TokenType::FALSE => Ok(Value::Bool(false)),
            TokenType::TRUE => Ok(Value::Bool(true)),
            TokenType::NUMBER { literal } => Ok(Value::Number(literal.0)),
            TokenType::STRING { literal } => Ok(Value::String(literal.clone())),
            _ => Err("cast TokenType to Value error".to_string()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;
    fn sub(self, rhs: Value) -> Self::Output {
        if let (Value::Number(l), Value::Number(r)) = (self, rhs) {
            return Ok(Value::Number(l - r));
        }
        return Err("Value cannot sub".to_string());
    }
}

impl Add for Value {
    type Output = Result<Value, String>;
    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            _ => Err("Value cannot add".to_string()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;
    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(l), Value::Number(r)) => match r {
                0.0 => Err("divide by zero".to_string()),
                _ => Ok(Value::Number(l / r)),
            },
            _ => Err("Value cannot div".to_string()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;
    fn mul(self, rhs: Value) -> Self::Output {
        if let (Value::Number(l), Value::Number(r)) = (self, rhs) {
            return Ok(Value::Number(l * r));
        }
        Err("Value cannot mul".to_string())
    }
}

impl PartialEq for Value {
    fn eq(&self, oth: &Self) -> bool {
        match &self {
            Value::Nil => match oth {
                Value::Nil => true,
                _ => false,
            },
            Value::String(s) => {
                if let Value::String(oth) = oth {
                    return s.eq(oth);
                }
                false
            }
            Value::Bool(b) => {
                if let Value::Bool(oth) = oth {
                    return b.eq(oth);
                }
                false
            }
            Value::Number(num) => {
                if let Value::Number(oth) = oth {
                    return num.eq(oth);
                }
                false
            }
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(num) => write!(f, "{}", num),
            Value::String(s) => write!(f, "{}", s),
            Value::Callable(c) => write!(f, "{}", c.to_string()),
        }
    }
}

pub struct FuncValue {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,                   // Stmt::Block
    closure: Rc<RefCell<Environment>>, // the env when the function is declared, not when it's called.
}

impl FuncValue {
    pub fn from(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> FuncValue {
        FuncValue {
            name,
            params,
            body,
            closure,
        }
    }
}

impl Callable for FuncValue {
    fn call(&self, mut i: Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let previous = i.env.clone();
        i.env = Rc::new(RefCell::new(Environment::from(&self.closure)));
        for idx in 0..self.params.len() {
            i.env
                .borrow_mut()
                .define(self.params[idx].lexeme.clone(), args[idx].clone());
        }
        let res = i.execute_stmts(&self.body)?;
        i.env = previous;
        // Lox 定义一个函数没有返回值时，默认返回 nil.
        Ok(res.0)
    }
    fn arity(&self) -> usize {
        self.params.len()
    }
    fn to_string(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}

pub struct Clock {}

impl Callable for Clock {
    fn call(&self, _i: Interpreter, _args: Vec<Value>) -> Result<Value, Error> {
        Ok(Value::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        ))
    }
    fn arity(&self) -> usize {
        0
    }
    fn to_string(&self) -> String {
        String::from("<native fn>")
    }
}
