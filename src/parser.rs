use crate::token::{self, Token, TokenType};
use std::fmt::Display;

#[derive(Debug)]
pub enum Expr {
    Binary(
        Box<Expr>, // left
        Token,     // operator
        Box<Expr>, // right
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
            Expr::Binary(left, operator, right) => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator.lexeme, right),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Assign(name, expr) => write!(f, "{} = {}", name, expr),
        }
    }
}

impl Expr {
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

pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var((String, Option<Expr>)),
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
        if self.match_(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
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
        if self.match_(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
        .map_err(|err| {
            self._synchronize();
            err
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        if self.match_(&[TokenType::Equal]) {
            let initializer = Some(self.expression()?);
            self.consume(
                TokenType::Semicolon,
                "Expect ';' after variable declaration.",
            )?;
            Ok(Stmt::Var((name.lexeme, initializer)))
        } else {
            self.consume(
                TokenType::Semicolon,
                "Expect ';' after variable declaration.",
            )?;
            Ok(Stmt::Var((name.lexeme, None)))
        }
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.match_(&[TokenType::Equal]) {
            let _equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::assign(name, value));
            } else {
                eprintln!("Invalid assignment target: {}", expr);
            }
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
            self.primary()
        }
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
