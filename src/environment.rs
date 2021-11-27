use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::RuntimeError;
use crate::object::Object;
use crate::token::Token;

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Rc<Object>>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }
    }

    pub fn define(&mut self, name: &str, value: Rc<Object>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Rc<Object>, RuntimeError> {
        match self.values.get(&name.lexeme) {
            None => match &self.enclosing {
                None => {
                    let message = format!("Undefined variable: '{}'.", name.lexeme);
                    Err(RuntimeError::new(name.clone(), message))
                }
                Some(enclosing) => enclosing.borrow().get(name),
            },
            Some(result) => Ok(result.clone()),
        }
    }

    pub fn get_at(
        environment: Rc<RefCell<Environment>>,
        distance: usize,
        name: &Token,
    ) -> Result<Rc<Object>, RuntimeError> {
        Environment::ancestor(environment, distance)
            .borrow()
            .values
            .get(&name.lexeme)
            .cloned()
            .ok_or_else(|| RuntimeError::new(name.clone(), "TODO: couldn't get_at"))
    }

    fn ancestor(
        mut environment: Rc<RefCell<Environment>>,
        distance: usize,
    ) -> Rc<RefCell<Environment>> {
        for _ in 0..distance {
            let refcell = environment.borrow().enclosing.as_ref().unwrap().clone();
            let _ = std::mem::replace(&mut environment, refcell);
        }

        environment
    }

    pub fn assign(&mut self, name: &Token, value: Rc<Object>) -> Result<(), RuntimeError> {
        if let Entry::Occupied(mut e) = self.values.entry(name.lexeme.clone()) {
            e.insert(value);
            Ok(())
        } else {
            match &mut self.enclosing {
                None => {
                    let message = format!("Undefined variable: '{}'.", name.lexeme);
                    Err(RuntimeError::new(name.clone(), message))
                }
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            }
        }
    }

    pub fn assign_at(
        environment: Rc<RefCell<Environment>>,
        distance: usize,
        name: &Token,
        value: Rc<Object>,
    ) -> Result<(), RuntimeError> {
        Environment::ancestor(environment, distance)
            .borrow_mut()
            .values
            .insert(name.lexeme.clone(), value)
            .map(|_| ())
            .ok_or_else(|| RuntimeError::new(name.clone(), "TODO: couldn't assign_at"))
    }
}
