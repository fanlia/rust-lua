mod lexer;
mod parser;
mod value;
mod vm;

use lexer::Lexer;
use parser::Parser;
use std::io::{self, Write};
use vm::Vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        run_file(&args[1]);
    } else {
        run_repl();
    }
}

fn run_file(filename: &str) {
    let source = std::fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Error: Could not read file {}", filename);
        std::process::exit(1);
    });

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();

    let mut vm = Vm::new();
    vm.execute(stmts);
}

fn run_repl() {
    let mut vm = Vm::new();

    println!("Lua Interpreter in Rust");
    println!("Type 'exit' to quit");

    loop {
        print!("> > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input == "exit" || input == "quit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let stmts = parser.parse();

        let result = vm.execute(stmts);
        if result != value::Value::Nil {
            println!("{}", result);
        }
    }
}
