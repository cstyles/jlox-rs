use std::io::{BufRead, Write};

mod environment;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod token;

use interpreter::Interpreter;
use parser::Parser;
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
    let mut interpreter = Interpreter::new();
    print_prompt();

    while stdin.read_line(&mut line).is_ok() {
        let trimmed = String::from(line.trim());
        let _ = run(&mut interpreter, trimmed);
        // run(line.clone());

        line.clear();
        print_prompt();
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().expect("error flushing stdout");
}

fn run(interpreter: &mut Interpreter, source: String) -> Result<(), ()> {
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
    for statement in statements {
        if let Err(err) = interpreter.evaluate_stmt(statement) {
            eprintln!("{}", err);
        }
    }

    Ok(())
}

fn run_file(filename: &str) {
    let program = std::fs::read_to_string(filename).expect("error reading file");
    let mut interpreter = Interpreter::new();
    if run(&mut interpreter, program).is_err() {
        std::process::exit(65);
    }
}
