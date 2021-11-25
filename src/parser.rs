use crate::token::{self, Token, TokenType};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Expr {
    Logical(
        Box<Expr>, // left
        Token,     // operator
        Box<Expr>, // right
    ),
    Binary(
        Box<Expr>, // left
        Token,     // operator
        Box<Expr>, // right
    ),
    Call(
        Box<Expr>, // callee
        Token,     // closing parenthesis
        Vec<Expr>, // arguments
    ),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(
        Token,     // operator
        Box<Expr>, // right
    ),
    Variable(Token),
    Assign(
        Token,     // identifier
        Box<Expr>, // value
    ),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Logical(left, operator, right) | Expr::Binary(left, operator, right) => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator.lexeme, right),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Assign(name, expr) => write!(f, "{} = {}", name, expr),
            Expr::Call(callee, _paren, args) => write!(f, "{}({:?})", callee, args),
        }
    }
}

impl Expr {
    fn logical(left: Self, operator: Token, right: Self) -> Self {
        Self::Logical(Box::new(left), operator, Box::new(right))
    }

    fn binary(left: Self, operator: Token, right: Self) -> Self {
        Self::Binary(Box::new(left), operator, Box::new(right))
    }

    fn grouping(expr: Self) -> Self {
        Self::Grouping(Box::new(expr))
    }

    fn unary(operator: Token, right: Self) -> Self {
        Self::Unary(operator, Box::new(right))
    }

    fn assign(name: Token, expr: Self) -> Self {
        Self::Assign(name, Box::new(expr))
    }

    fn call(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
        Self::Call(Box::new(callee), paren, arguments)
    }
}

fn _test_pretty_print() {
    let expr = Expr::binary(
        Expr::unary(
            Token::new(TokenType::Minus, "-".into(), token::Literal::None, 1),
            Expr::Literal(token::Literal::Number(123.0)),
        ),
        Token::new(TokenType::Star, "*".into(), token::Literal::None, 1),
        Expr::grouping(Expr::Literal(token::Literal::Number(45.67))),
    );

    println!("{}", expr);
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    If(
        Expr,              // condition
        Box<Stmt>,         // then branch
        Box<Option<Stmt>>, // else branch
    ),
    Expression(Expr),
    Function(
        Token,      // name
        Vec<Token>, // parameters
        Vec<Stmt>,  // body
    ),
    Print(Expr),
    Return(
        Token,        // keyword
        Option<Expr>, // return valuue
    ),
    Var(String, Option<Expr>),
    While(
        Expr,      // condition
        Box<Stmt>, // body
    ),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration().ok()?);
        }

        Some(statements)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_(&[TokenType::Semicolon]) {
            None
        } else if self.match_(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let body = self.statement()?;
        let body = match increment {
            Some(expr) => Stmt::Block(vec![body, Stmt::Expression(expr)]),
            None => body,
        };

        let condition = condition.unwrap_or(Expr::Literal(token::Literal::True));
        let body = Stmt::While(condition, Box::new(body));

        let body = match initializer {
            Some(stmt) => Stmt::Block(vec![stmt, body]),
            None => body,
        };

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_(&[TokenType::Fun]) {
            self.function(FunctionKind::Function)
        } else if self.match_(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
        .map_err(|err| {
            self._synchronize();
            err
        })
    }

    fn function(&mut self, kind: FunctionKind) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    print_error(self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;

        Ok(Stmt::Function(name, parameters, body))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        if self.match_(&[TokenType::Equal]) {
            let initializer = Some(self.expression()?);
            self.consume(
                TokenType::Semicolon,
                "Expect ';' after variable declaration.",
            )?;
            Ok(Stmt::Var(name.lexeme, initializer))
        } else {
            self.consume(
                TokenType::Semicolon,
                "Expect ';' after variable declaration.",
            )?;
            Ok(Stmt::Var(name.lexeme, None))
        }
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::assign(name, value));
            } else {
                print_error(equals, &format!("Invalid assignment target: {}", expr));
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;

            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;

            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right: Expr = self.comparison()?;

            expr = Expr::binary(expr, operator.clone(), right);
        }

        Ok(expr)
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous();

        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword, value))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(expr))
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }

        // self.peek().map_or(false, |tt| tt == token_type)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    // Can return Option<&Token> instead
    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            Ok(Expr::unary(operator, right))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, something: Expr) -> Result<Expr, ParseError> {
        let mut arguments = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    print_error(self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::call(something, paren, arguments))
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_(&[TokenType::False]) {
            Ok(Expr::Literal(token::Literal::False))
        } else if self.match_(&[TokenType::True]) {
            Ok(Expr::Literal(token::Literal::True))
        } else if self.match_(&[TokenType::Nil]) {
            Ok(Expr::Literal(token::Literal::Nil))
        } else if self.match_(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::Literal(self.previous().literal))
        } else if self.match_(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous()))
        } else if self.match_(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

            Ok(Expr::grouping(expr))
        } else {
            print_error(self.peek(), "Expect expression.");
            Err(ParseError)
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            print_error(self.peek(), message);
            Err(ParseError)
        }
    }

    fn _synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wow")
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
enum FunctionKind {
    Function,
    #[allow(unused)]
    Method,
}

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FunctionKind::Function => "function",
                FunctionKind::Method => "method",
            }
        )
    }
}

fn print_error(token: Token, message: &str) {
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
