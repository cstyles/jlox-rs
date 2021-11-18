use std::io::{BufRead, Write};

mod parser;
mod scanner;
mod token;

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
