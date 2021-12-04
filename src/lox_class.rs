use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::{Callable, LoxFunction};
use crate::interpreter::{Interpreter, RuntimeError};
use crate::lox_instance::LoxInstance;
use crate::object::Object;
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
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
        interpreter: &mut Interpreter,
        paren: &Token,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeError> {
        let instance = LoxInstance::new(self.clone());

        let initializer = self.find_method("init").cloned();
        if let Some(initializer) = initializer {
            let _ = initializer
                .bind(&instance)
                .call(interpreter, paren, arguments);
        }

        Ok(Rc::new(Object::Instance(RefCell::new(instance))))
    }

    fn arity(&self) -> usize {
        self.find_method("init").map_or(0, |init| init.arity())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
