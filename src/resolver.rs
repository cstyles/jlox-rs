use std::collections::HashMap;
use std::ops::Deref;

use crate::interpreter::Interpreter;
use crate::parser::Expr;
use crate::parser::FunctionKind;
use crate::parser::Stmt;
use crate::token::Token;
use crate::token::TokenType;

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>, // TODO: HashSet instead?
    current_function: Option<FunctionKind>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: None,
        }
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Var(name, initializer) => {
                self.declare(name);

                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer);
                }

                self.define(name);
            }
            Stmt::Block(statements) => {
                self.begin_scope();
                for statement in statements {
                    self.resolve_statement(statement);
                }
                self.end_scope();
            }
            Stmt::Class(name, methods) => {
                self.declare(&name.lexeme);
                self.define(&name.lexeme);

                // TODO: Can we push (name, params, body) into `methods` in `Parser`
                // so we don't need to check if the method is actually a Function Stmt?
                for method in methods {
                    if let Stmt::Function(_name, parameters, body) = method {
                        let declaration = FunctionKind::Method;
                        self.resolve_function(parameters, body, declaration);
                    } else {
                        print_error(name, "Method wasn't a function.");
                    }
                }
            }
            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch.deref() {
                    self.resolve_statement(else_branch);
                }
            }
            Stmt::Expression(expr) => self.resolve_expression(expr),
            Stmt::Function(name, parameters, body) => {
                self.declare(&name.lexeme);
                self.define(&name.lexeme);

                self.resolve_function(parameters, body, FunctionKind::Function);
            }
            Stmt::Print(expr) => self.resolve_expression(expr),
            Stmt::Return(keyword, return_value) => {
                if self.current_function.is_none() {
                    print_error(keyword, "Can't return from top-level code.");
                }

                if let Some(return_value) = return_value {
                    self.resolve_expression(return_value);
                }
            }
            Stmt::While(condition, body) => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Logical(left, _operator, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Binary(left, _operator, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call(callee, _paren, arguments) => {
                self.resolve_expression(callee);
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
            Expr::Grouping(expr) => self.resolve_expression(expr),
            Expr::Literal(_literal) => {} // no-op
            Expr::Unary(_operator, right) => self.resolve_expression(right),
            Expr::Variable(name) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(false) = scope.get(&name.lexeme) {
                        print_error(name, "Can't read local variable in its own initializer.");
                    }
                }

                self.resolve_local(expr, name);
            }
            Expr::Assign(identifier, value) => {
                self.resolve_expression(value);
                self.resolve_local(value, identifier);
            }
            Expr::Get(object, _name) => self.resolve_expression(object),
            Expr::Set(object, _name, value) => {
                self.resolve_expression(value);
                self.resolve_expression(object);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                eprintln!("Already a variable called '{}' in this scope.", name);
            }
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                let depth = self.scopes.len() - 1 - i;
                self.interpreter.resolve(expr, depth);
                return;
            }
        }
    }

    fn resolve_function(&mut self, params: &[Token], body: &[Stmt], kind: FunctionKind) {
        // Store current_function for later
        let previous = std::mem::replace(&mut self.current_function, Some(kind));

        self.begin_scope();

        for param in params {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        self.resolve_statements(body);
        self.end_scope();

        // Restore previous current_function
        let _ = std::mem::replace(&mut self.current_function, previous);
    }
}

fn print_error(token: &Token, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, "at end", message);
    } else {
        report(token.line, &format!("at '{}'", token.lexeme), message);
    }
}

// TODO: dup
fn report(line_number: usize, location: &str, message: &str) {
    println!("[line {}] Error {}: {}", line_number, location, message);
}
