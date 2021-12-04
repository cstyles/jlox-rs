use crate::environment::Environment;
use crate::interpreter::{Control, Interpreter, RuntimeError};
use crate::lox_instance::LoxInstance;
use crate::object::Object;
use crate::parser::Stmt;
use crate::token::{Literal, Token, TokenType};

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::SystemTime;

pub trait Callable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError>;

    fn arity(&self) -> usize;

    fn name(&self) -> &str;

    fn check_arity(&self, paren: &Token, arguments: &[Rc<Object>]) -> Result<(), RuntimeError> {
        if arguments.len() != self.arity() {
            let message = format!(
                "Expected {} arguments but got {}.",
                self.arity(),
                arguments.len()
            );
            return Err(RuntimeError::new(paren.clone(), message));
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
        paren: &Token,
        _arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => Ok(Rc::new(Object::from(duration.as_secs() as f64))),
            Err(err) => Err(RuntimeError::new(paren.clone(), err.to_string())),
        }
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: String,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: String,
        parameters: &[Token],
        body: &[Stmt],
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        let parameters = parameters.to_vec();
        let body = body.to_vec();

        Self {
            name,
            parameters,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn bind(self, instance: &LoxInstance) -> LoxFunction {
        let instance = Rc::new(Object::Instance(RefCell::new(instance.clone())));
        self.closure.borrow_mut().define("this", instance);

        LoxFunction::new(
            self.name,
            &self.parameters,
            &self.body,
            self.closure,
            self.is_initializer,
        )
    }
}

impl Callable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        self.check_arity(paren, &arguments)?;

        let mut environment = Environment::from_enclosing(self.closure.clone());

        for (param, arg) in self.parameters.iter().zip(arguments) {
            environment.define(&param.lexeme, arg);
        }

        match interpreter.execute_block(&self.body, environment) {
            Ok(()) => {
                if self.is_initializer {
                    // TODO: this is hacky
                    let this_token = Token::new(
                        TokenType::This,
                        "this".into(),
                        Literal::String("this".into()),
                        paren.line,
                        paren.column,
                    );
                    Environment::get_at(self.closure.clone(), 0, &this_token)
                } else {
                    // TODO: check if this can ever run
                    Ok(Rc::new(Object::Nil))
                }
            }
            Err(Control::Return(value)) => {
                if self.is_initializer {
                    // TODO: this is hacky
                    let this_token = Token::new(
                        TokenType::This,
                        "this".into(),
                        Literal::String("this".into()),
                        paren.line,
                        paren.column,
                    );
                    Environment::get_at(self.closure.clone(), 0, &this_token)
                } else {
                    Ok(value)
                }
            }
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
