use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Deref, Not};
use std::rc::Rc;

use crate::callable::{Clock, LoxFunction};
use crate::environment::Environment;
use crate::object::Object;
use crate::parser::{Expr, Stmt};
use crate::token::{Token, TokenType};

#[derive(Default)]
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut i: Self = Default::default();

        // Define native function `clock` in the global scope
        i.globals
            .borrow_mut()
            .define("clock", Rc::new(Object::Callable(Box::new(Clock {}))));

        // Alias the initial environment to the globals environment
        i.environment = i.globals.clone();

        i
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Rc<Object>, RuntimeError> {
        match expr {
            Expr::Logical(left, operator, right) => self.evaluate_logical(left, operator, right),
            Expr::Binary(left, operator, right) => self.evaluate_binary(left, operator, right),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(lit) => Ok(Rc::new(Object::from(lit))),
            Expr::Unary(operator, expr) => self.evaluate_unary(operator, expr),
            Expr::Variable(name) => self.lookup_variable(name, expr),
            Expr::Assign(name, expr) => self.assign_variable(name, expr),
            Expr::Call(callee, paren, args) => self.evaluate_call(callee, paren, args),
        }
    }

    fn evaluate_call(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
    ) -> Result<Rc<Object>, RuntimeError> {
        let callee = self.evaluate(callee)?;

        let arguments: Result<Vec<Rc<Object>>, RuntimeError> =
            arguments.iter().map(|arg| self.evaluate(arg)).collect();
        let arguments = arguments?;

        match callee.deref() {
            Object::Callable(fun) => fun.call(self, paren, arguments),
            _ => Err(RuntimeError::new(
                paren.clone(),
                format!("'{}' is not callable", callee),
            )),
        }
    }

    pub fn evaluate_stmt(&mut self, stmt: &Stmt) -> Result<(), Control> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::If(expr, then_branch, else_branch) => {
                if self.evaluate(expr)?.is_truthy() {
                    self.evaluate_stmt(&**then_branch)?;
                } else if let Some(else_branch) = &**else_branch {
                    self.evaluate_stmt(else_branch)?;
                }

                Ok(())
            }
            Stmt::Print(expr) => {
                let object = self.evaluate(expr)?;
                println!("{}", object);
                Ok(())
            }
            Stmt::Var(name, expr) => match expr {
                Some(expr) => {
                    let value = self.evaluate(expr)?;
                    self.environment.borrow_mut().define(name, value);
                    Ok(())
                }
                None => {
                    self.environment
                        .borrow_mut()
                        .define(name, Rc::new(Object::Nil));
                    Ok(())
                }
            },
            Stmt::Block(statements) => {
                let environment = Environment::from_enclosing(self.environment.clone());
                self.execute_block(statements, environment)
            }
            Stmt::While(condition, body) => {
                while self.evaluate(condition)?.is_truthy() {
                    self.evaluate_stmt(body)?;
                }

                Ok(())
            }
            Stmt::Function(name, params, body) => {
                let function =
                    LoxFunction::new(name.lexeme.clone(), params, body, self.environment.clone());
                let object = Rc::new(Object::Callable(Box::new(function)));
                self.environment.borrow_mut().define(&name.lexeme, object);
                Ok(())
            }
            Stmt::Return(_keyword, None) => Err(Control::Return(Rc::new(Object::Nil))),
            Stmt::Return(_keyword, Some(expr)) => Err(Control::Return(self.evaluate(expr)?)),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), Control> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));

        for statement in statements {
            if let Err(err) = self.evaluate_stmt(statement) {
                self.environment = previous;
                return Err(err);
            }
        }

        self.environment = previous;

        Ok(())
    }

    fn evaluate_unary(
        &mut self,
        operator: &Token,
        right: &Expr,
    ) -> Result<Rc<Object>, RuntimeError> {
        let right = self.evaluate(right)?;

        match (operator.token_type, &right) {
            (TokenType::Minus, _) => (-right.deref()).context(operator, "Operand must be a number"),
            (TokenType::Bang, _) => Ok(!right.deref()),
            _ => unreachable!("unary expression with bad operator: {}", operator),
        }
    }

    fn evaluate_logical(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Rc<Object>, RuntimeError> {
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
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Rc<Object>, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => (left.deref() - &right).assert_numbers(operator),
            TokenType::Slash => (left.deref() / &right).assert_numbers(operator),
            TokenType::Star => (left.deref() * &right).assert_numbers(operator),
            TokenType::Plus => (left.deref() + &right)
                .context(operator, "Operands must be two numbers or two strings."),
            TokenType::Greater => left
                .partial_cmp(&right)
                .map(|o| Rc::new(o.is_gt().into()))
                .assert_numbers(operator),
            TokenType::GreaterEqual => left
                .partial_cmp(&right)
                .map(|o| Rc::new(o.is_ge().into()))
                .assert_numbers(operator),
            TokenType::Less => left
                .partial_cmp(&right)
                .map(|o| Rc::new(o.is_lt().into()))
                .assert_numbers(operator),
            TokenType::LessEqual => left
                .partial_cmp(&right)
                .map(|o| Rc::new(o.is_le().into()))
                .assert_numbers(operator),
            TokenType::BangEqual => Ok(Rc::new((*left).eq(&right).not().into())),
            TokenType::EqualEqual => Ok(Rc::new((*left).eq(&right).into())),
            _ => unreachable!("not possible operands: {:?}, {:?}", left, right),
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Rc<Object>, RuntimeError> {
        match self.locals.get(expr) {
            Some(distance) => Environment::get_at(self.environment.clone(), *distance, name),
            None => self.globals.borrow().get(name),
        }
    }

    fn assign_variable(&mut self, name: &Token, expr: &Expr) -> Result<Rc<Object>, RuntimeError> {
        let value = self.evaluate(expr)?;

        match self.locals.get(expr) {
            Some(distance) => {
                Environment::assign_at(self.environment.clone(), *distance, name, value.clone())?
            }
            None => self.globals.borrow_mut().assign(name, value.clone())?,
        };

        Ok(value)
    }
}

#[derive(Debug)]
pub enum Control {
    Return(Rc<Object>),
    Error(RuntimeError),
}

impl Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::Return(value) => Display::fmt(value, f),
            Control::Error(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Control {}

impl From<RuntimeError> for Control {
    fn from(err: RuntimeError) -> Self {
        Control::Error(err)
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

// impl std::error::Error for Control {}

trait Contextable: Sized {
    fn context(self, operator: &Token, message: impl ToString) -> Result<Rc<Object>, RuntimeError>;

    fn assert_numbers(self, operator: &Token) -> Result<Rc<Object>, RuntimeError> {
        self.context(operator, "Operands must be numbers.")
    }
}

impl Contextable for Option<Rc<Object>> {
    fn context(self, operator: &Token, message: impl ToString) -> Result<Rc<Object>, RuntimeError> {
        self.ok_or_else(|| RuntimeError::new(operator.clone(), message))
    }
}
