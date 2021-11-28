use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::Callable;
use crate::interpreter::{Interpreter, RuntimeError};
use crate::lox_instance::LoxInstance;
use crate::object::Object;
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Callable for LoxClass {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _paren: &Token,
        _arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        let instance = LoxInstance::new(self.clone());
        Ok(Rc::new(Object::Instance(RefCell::new(instance))))
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        &self.name
    }
}