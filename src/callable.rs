use crate::environment::Environment;
use crate::interpreter::{Control, Interpreter, RuntimeError};
use crate::object::Object;
use crate::parser::Stmt;
use crate::token::Token;

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::SystemTime;

pub trait Callable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: Token,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError>;

    fn arity(&self) -> usize;

    fn name(&self) -> &str;

    fn check_arity(&self, paren: Token, arguments: &[Rc<Object>]) -> Result<(), RuntimeError> {
        if arguments.len() != self.arity() {
            let message = format!(
                "Expected {} arguments but got {}.",
                self.arity(),
                arguments.len()
            );
            return Err(RuntimeError::new(paren, message));
        }

        Ok(())
    }
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}

pub struct Clock {}

impl Callable for Clock {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        paren: Token,
        _arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => Ok(Rc::new(Object::from(duration.as_secs() as f64))),
            Err(_) => Err(RuntimeError::new(paren, String::from("clock failed!"))),
        }
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }
}

pub struct LoxFunction {
    name: String,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: String,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            closure,
        }
    }
}

impl Callable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: Token,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        self.check_arity(paren, &arguments)?;

        let mut environment = Environment::from_enclosing(self.closure.clone());

        for (param, arg) in self.parameters.iter().zip(arguments) {
            environment.define(param.lexeme.clone(), arg);
        }

        match interpreter.execute_block(self.body.clone(), environment) {
            Ok(()) => Ok(Rc::new(Object::Nil)),
            Err(Control::Return(value)) => Ok(value),
            Err(Control::Error(err)) => Err(err),
        }
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}