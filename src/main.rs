use std::{
    fmt::Display,
    io::{BufRead, Write},
};

mod scanner;
mod token;

use scanner::Scanner;
use token::{Token, TokenType};

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

trait Visitor<R> {}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.len() {
        0 => run_prompt(),
        1 => run_file(args.first().unwrap()),
        _ => {
            eprintln!("Usage: jlox [script]");
            std::process::exit(64);
        }
    };
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut line = String::with_capacity(100);
    print_prompt();

    while stdin.read_line(&mut line).is_ok() {
        let trimmed = String::from(line.trim());
        run(trimmed);
        // run(line.clone());

        line.clear();
        print_prompt();
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().expect("error flushing stdout");
}

fn run(source: String) -> Result<(), ()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().expect("scan error");

    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}

fn run_file(filename: &str) {
    let program = std::fs::read_to_string(filename).expect("error reading file");
    if run(program).is_err() {
        std::process::exit(65);
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
