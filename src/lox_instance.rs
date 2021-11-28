use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::lox_class::LoxClass;
use crate::object::Object;
use crate::token::Token;

#[derive(Debug)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: HashMap<String, Rc<Object>>,
}

impl LoxInstance {
    pub fn new(klass: LoxClass) -> Self {
        Self {
            klass,
            fields: HashMap::default(),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Rc<Object>> {
        self.fields.get(&name.lexeme).cloned().or_else(|| {
            // If no field found, check for a method on the class
            self.klass
                .find_method(&name.lexeme)
                .cloned()
                .map(|method| Rc::new(Object::Callable(Box::new(method))))
        })
    }

    pub fn set(&mut self, name: &Token, value: Rc<Object>) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}
