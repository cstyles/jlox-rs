use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, Write},
};

#[derive(Default)]
struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    line_number: usize,
    start: usize,
    current: usize,
    had_error: bool,
    keywords: HashMap<String, TokenType>, // TODO: make static
}

impl Scanner {
    fn new(source: String) -> Self {
        let keywords = [
            ("and".to_string(), TokenType::And),
            ("class".to_string(), TokenType::Class),
            ("else".to_string(), TokenType::Else),
            ("false".to_string(), TokenType::False),
            ("for".to_string(), TokenType::For),
            ("fun".to_string(), TokenType::Fun),
            ("if".to_string(), TokenType::If),
            ("nil".to_string(), TokenType::Nil),
            ("or".to_string(), TokenType::Or),
            ("print".to_string(), TokenType::Print),
            ("return".to_string(), TokenType::Return),
            ("super".to_string(), TokenType::Super),
            ("this".to_string(), TokenType::This),
            ("true".to_string(), TokenType::True),
            ("var".to_string(), TokenType::Var),
            ("while".to_string(), TokenType::While),
        ]
        .into();

        Self {
            source: source.chars().collect(),
            keywords,
            ..Default::default()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>, ()> {
        // let chars: Vec<char> = self.source.chars().collect();
        while !self.is_at_end() {
            self.start = self.current;

            match self.advance() {
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                '{' => self.add_token(TokenType::LeftBrace),
                '}' => self.add_token(TokenType::RightBrace),
                ',' => self.add_token(TokenType::Comma),
                '.' => self.add_token(TokenType::Dot),
                '-' => self.add_token(TokenType::Minus),
                '+' => self.add_token(TokenType::Plus),
                ';' => self.add_token(TokenType::Semicolon),
                '*' => self.add_token(TokenType::Star),
                '!' => {
                    if self.match_('=') {
                        self.add_token(TokenType::BangEqual);
                    } else {
                        self.add_token(TokenType::Bang);
                    }
                }
                '=' => {
                    if self.match_('=') {
                        self.add_token(TokenType::EqualEqual);
                    } else {
                        self.add_token(TokenType::Equal);
                    }
                }
                '<' => {
                    if self.match_('=') {
                        self.add_token(TokenType::LessEqual);
                    } else {
                        self.add_token(TokenType::Less);
                    }
                }
                '>' => {
                    if self.match_('=') {
                        self.add_token(TokenType::GreaterEqual);
                    } else {
                        self.add_token(TokenType::Greater);
                    }
                }
                '/' => {
                    if self.match_('/') {
                        while self.peek().map_or(false, |c| *c != '\n') {
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenType::Slash);
                    }
                }
                ' ' | '\r' | '\t' => {}
                '\n' => self.line_number += 1,
                '"' => self.string(),
                '0'..='9' => self.digit(),
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
                _ => {
                    print_error(self.line_number, String::from("Unexpected character."));
                    self.had_error = true;
                }
            };
        }

        self.add_token(TokenType::Eof);

        if self.had_error {
            Err(())
        } else {
            Ok(self.tokens.clone())
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, Literal::None)
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        let token = Token {
            token_type,
            lexeme: text.iter().collect(),
            literal,
            line: self.line_number,
        };
        self.tokens.push(token);
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> Option<&char> {
        self.source.get(self.current)
    }

    fn peek_next(&self) -> Option<&char> {
        self.source.get(self.current + 1)
    }

    fn string(&mut self) {
        // Chomp until we reach the closing quote or the end of the input
        while let Some(c) = self.peek() {
            match *c {
                '"' => break,
                '\n' => self.line_number += 1,
                _ => {}
            }

            self.advance();
        }

        // Raise an error if the string was unterminated
        if self.is_at_end() {
            print_error(self.line_number, "Unterminated string".to_string());
            return;
        }

        // Chomp the closing "
        self.advance();

        // Trim the surrounding quotes
        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token_with_literal(TokenType::String, Literal::String(value));
    }

    fn digit(&mut self) {
        // Munch as many numeric characters as possible
        while let Some('0'..='9') = self.peek() {
            self.advance();
        }

        // If we hit a dot followed by another number...
        if self.peek() == Some(&'.') && matches!(self.peek_next(), Some('0'..='9')) {
            // consume the dot...
            self.advance();

            // and any numbers after it
            while let Some('0'..='9') = self.peek() {
                self.advance();
            }
        }

        // Convert string representation into an f64
        let value: String = self.source[self.start..self.current].iter().collect();
        let value: f64 = value.parse().expect("must be numeric");
        self.add_token_with_literal(TokenType::Number, Literal::Number(value));
    }

    fn identifier(&mut self) {
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.peek() {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = *self.keywords.get(&text).unwrap_or(&TokenType::Identifier);
        self.add_token(token_type);
    }
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Literal,
    line: usize,
}

#[derive(Debug, Clone)]
enum Literal {
    None,
    String(String),
    Number(f64),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)] // TODO: remove me?
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

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

fn print_error(line_number: usize, message: String) {
    report(line_number, "", message);
}

fn report(line_number: usize, location: &str, message: String) {
    println!("[line {}] Error {}: {}", line_number, location, message);
}
