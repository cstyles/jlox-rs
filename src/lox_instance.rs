use std::fmt::Display;
use std::rc::Rc;

use crate::lox_class::LoxClass;

#[derive(Debug)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self { klass }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}
