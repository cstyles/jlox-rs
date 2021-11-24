use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Not;
use std::rc::Rc;

use crate::environment::Environment;
use crate::object::Object;
use crate::parser::{Expr, Stmt};
use crate::token::{Token, TokenType};

#[derive(Default)]
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Object, RuntimeError> {
        match expr {
            Expr::Logical(left, operator, right) => self.evaluate_logical(*left, operator, *right),
            Expr::Binary(left, operator, right) => self.evaluate_binary(*left, operator, *right),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Literal(lit) => Ok(Object::from(lit)),
            Expr::Unary(operator, expr) => self.evaluate_unary(operator, *expr),
            Expr::Variable(name) => self.environment.borrow().get(name),
            Expr::Assign(name, expr) => {
                let value = self.evaluate(*expr)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
        }
    }

    pub fn evaluate_stmt(&mut self, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::If(expr, then_branch, else_branch) => {
                if self.evaluate(expr)?.is_truthy() {
                    self.evaluate_stmt(*then_branch)?;
                } else if let Some(else_branch) = *else_branch {
                    self.evaluate_stmt(else_branch)?;
                }

                Ok(())
            }
            Stmt::Print(expr) => {
                let object = self.evaluate(expr)?;
                println!("{}", object);
                Ok(())
            }
            Stmt::Var((name, expr)) => match expr {
                Some(expr) => {
                    let value = self.evaluate(expr)?;
                    self.environment.borrow_mut().define(name, value);
                    Ok(())
                }
                None => {
                    self.environment.borrow_mut().define(name, Object::Nil);
                    Ok(())
                }
            },
            Stmt::Block(statements) => {
                let environment = Environment::from_enclosing(self.environment.clone());
                self.execute_block(statements, environment)
            }
            Stmt::While(condition, body) => {
                let body = *body;
                while self.evaluate(condition.clone())?.is_truthy() {
                    self.evaluate_stmt(body.clone())?;
                }

                Ok(())
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: Environment,
    ) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        for statement in statements {
            self.evaluate_stmt(statement)?;
        }

        self.environment = previous;

        Ok(())
    }

    fn evaluate_unary(&mut self, operator: Token, right: Expr) -> Result<Object, RuntimeError> {
        let right = self.evaluate(right)?;

        match (operator.token_type, &right) {
            (TokenType::Minus, _) => (-right).context(operator, "Operand must be a number"),
            (TokenType::Bang, _) => Ok(!right),
            _ => unreachable!("unary expression with bad operator: {}", operator),
        }
    }

    fn evaluate_logical(
        &mut self,
        left: Expr,
        operator: Token,
        right: Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;

        if operator.token_type == TokenType::And {
            if !left.is_truthy() {
                return Ok(left);
            }
        } else if left.is_truthy() {
            return Ok(left);
        }

        self.evaluate(right)
    }

    fn evaluate_binary(
        &mut self,
        left: Expr,
        operator: Token,
        right: Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

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
