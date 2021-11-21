use std::cmp::PartialOrd;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

use crate::token::Literal;

#[derive(Debug, PartialEq)]
pub enum Object {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl From<Literal> for Object {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::None => Self::Nil,
            Literal::String(string) => Self::String(string),
            Literal::Number(num) => Self::Number(num),
            Literal::False => Self::Boolean(false),
            Literal::True => Self::Boolean(true),
            Literal::Nil => Self::Nil,
        }
    }
}

impl From<bool> for Object {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<f64> for Object {
    fn from(num: f64) -> Self {
        Self::Number(num)
    }
}

impl From<String> for Object {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match *self {
            Object::Boolean(val) => val,
            Object::Nil => false,
            _ => true,
        }
    }
}

impl Add<Object> for Object {
    type Output = Option<Object>;

    fn add(self, right: Object) -> Self::Output {
        match (self, right) {
            (Object::Number(left), Object::Number(right)) => Some(Object::Number(left + right)),
            (Object::String(left), Object::String(right)) => Some(Object::String(left + &right)),
            _ => None,
        }
    }
}

impl Sub<Object> for Object {
    type Output = Option<Object>;

    fn sub(self, right: Object) -> Self::Output {
        (self, right)
            .as_numbers()
            .map(|(left, right)| Object::Number(left - right))
    }
}

impl Mul<Object> for Object {
    type Output = Option<Object>;

    fn mul(self, right: Object) -> Self::Output {
        (self, right)
            .as_numbers()
            .map(|(left, right)| Object::Number(left * right))
    }
}

impl Div<Object> for Object {
    type Output = Option<Object>;

    fn div(self, right: Object) -> Self::Output {
        (self, right)
            .as_numbers()
            .map(|(left, right)| Object::Number(left / right))
    }
}

impl Neg for Object {
    type Output = Option<Self>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(number) => Some(Self::Number(-number)),
            _ => None,
        }
    }
}

impl Not for Object {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Boolean(!self.is_truthy())
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.partial_cmp(right),
            (Self::String(left), Self::String(right)) => Some(left.cmp(right)),
            _ => None,
        }
    }
}

trait AsNumbers {
    fn as_numbers(&self) -> Option<(f64, f64)>;
}

impl AsNumbers for (Object, Object) {
    fn as_numbers(&self) -> Option<(f64, f64)> {
        match *self {
            (Object::Number(left), Object::Number(right)) => Some((left, right)),
            _ => None,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Number(num) => write!(f, "{}", num),
            Object::String(string) => write!(f, "{}", string),
        }
    }
}
