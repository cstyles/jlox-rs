use std::fmt::Display;
use crate::token::{self, Token, TokenType};

enum Expr {
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
