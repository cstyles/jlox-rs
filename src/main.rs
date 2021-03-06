use std::io::{BufRead, Write};

mod callable;
mod environment;
mod interpreter;
mod lox_class;
mod lox_instance;
mod object;
mod parser;
mod resolver;
mod scanner;
mod token;

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;

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
    let interpreter = Interpreter::new();
    let mut resolver = Resolver::new(interpreter);
    print_prompt();

    while stdin.read_line(&mut line).is_ok() {
        let trimmed = String::from(line.trim());
        let _ = run(&mut resolver, trimmed);
        // run(line.clone());

        line.clear();
        print_prompt();
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().expect("error flushing stdout");
}

fn run(resolver: &mut Resolver, source: String) -> Result<(), ()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().expect("scan error");

    // println!();
    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    // println!();
    // dbg!(&expr);

    let statements = statements.ok_or(())?;
    resolver.resolve_statements(&statements);

    for statement in statements {
        if let Err(err) = resolver.interpreter.evaluate_stmt(&statement) {
            eprintln!("{}", err);
        }
    }

    Ok(())
}

fn run_file(filename: &str) {
    let program = std::fs::read_to_string(filename).expect("error reading file");
    let interpreter = Interpreter::new();
    let mut resolver = Resolver::new(interpreter);
    if run(&mut resolver, program).is_err() {
        std::process::exit(65);
    }
}
