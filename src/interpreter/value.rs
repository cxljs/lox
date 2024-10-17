use core::fmt;
use std::ops::{Add, Div, Mul, Sub};

use crate::token::TokenType;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
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
}

impl TryFrom<&TokenType> for Value {
    type Error = String;
    fn try_from(t: &TokenType) -> Result<Value, Self::Error> {
        match t {
            TokenType::NIL => Ok(Value::Nil),
            TokenType::FALSE => Ok(Value::Bool(false)),
            TokenType::TRUE => Ok(Value::Bool(true)),
            TokenType::NUMBER { literal } => Ok(Value::Number(*literal)),
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(num) => write!(f, "{}", num),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}
