use std::fmt::Display;
use std::ops::Not;

use crate::object::Object;
use crate::parser::Expr;
use crate::token::{Token, TokenType};

pub fn evaluate(expr: Expr) -> Result<Object, RuntimeError> {
    match expr {
        Expr::Binary(left, operator, right) => evaluate_binary(*left, operator, *right),
        Expr::Grouping(expr) => evaluate(*expr),
        Expr::Literal(lit) => Ok(Object::from(lit)),
        Expr::Unary(operator, expr) => evaluate_unary(operator, *expr),
    }
}

fn evaluate_unary(operator: Token, right: Expr) -> Result<Object, RuntimeError> {
    let right = evaluate(right)?;

    match (operator.token_type, &right) {
        (TokenType::Minus, _) => (-right).context(operator, "Operand must be a number"),
        (TokenType::Bang, _) => Ok(!right),
        _ => unreachable!("unary expression with bad operator: {}", operator),
    }
}

fn evaluate_binary(left: Expr, operator: Token, right: Expr) -> Result<Object, RuntimeError> {
    let left = evaluate(left)?;
    let right = evaluate(right)?;

    match operator.token_type {
        TokenType::Minus => (left - right).assert_numbers(operator),
        TokenType::Slash => (left / right).assert_numbers(operator),
        TokenType::Star => (left * right).assert_numbers(operator),
        TokenType::Plus => {
            (left + right).context(operator, "Operands must be two numbers or two strings.")
        }
        TokenType::Greater => left
            .partial_cmp(&right)
            .map(|o| o.is_gt().into())
            .assert_numbers(operator),
        TokenType::GreaterEqual => left
            .partial_cmp(&right)
            .map(|o| o.is_ge().into())
            .assert_numbers(operator),
        TokenType::Less => left
            .partial_cmp(&right)
            .map(|o| o.is_lt().into())
            .assert_numbers(operator),
        TokenType::LessEqual => left
            .partial_cmp(&right)
            .map(|o| o.is_le().into())
            .assert_numbers(operator),
        TokenType::BangEqual => Ok(left.eq(&right).not().into()),
        TokenType::EqualEqual => Ok(left.eq(&right).into()),
        _ => unreachable!("not possible operands: {:?}, {:?}", left, right),
    }
}
#[derive(Debug)]
pub struct RuntimeError {
    operator: Token,
    message: String,
}

impl RuntimeError {
    pub fn new(operator: Token, message: impl ToString) -> Self {
        let message = message.to_string();
        Self { operator, message }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n[line {}]", self.message, self.operator.line)
    }
}

impl std::error::Error for RuntimeError {}

trait Contextable: Sized {
    fn context(self, operator: Token, message: impl ToString) -> Result<Object, RuntimeError>;

    fn assert_numbers(self, operator: Token) -> Result<Object, RuntimeError> {
        self.context(operator, "Operands must be numbers.")
    }
}

impl Contextable for Option<Object> {
    fn context(self, operator: Token, message: impl ToString) -> Result<Object, RuntimeError> {
        self.ok_or_else(|| RuntimeError::new(operator, message))
    }
}
