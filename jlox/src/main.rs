use clap::Parser;
use std::{
    fs,
    io::{self, BufRead},
};

use crate::interpreter::Interpreter;
use crate::scanner::scan_tokens;

pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod utils;

/// https://craftinginterpreters.com
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to jlox script to execute
    #[arg(num_args = 0..=1)]
    script: Option<String>,
}

fn main() {
    let args = Args::parse();
    let mut interpreter = Interpreter::new();

    match args.script {
        Some(script) => run_file(&mut interpreter, &script),
        None => run_prompt(&mut interpreter),
    };
}

fn run_file(interpreter: &mut Interpreter, script_path: &str) {
    let Ok(contents) = fs::read_to_string(script_path) else {
        panic!("Failed to find script");
    };

    run(interpreter, &contents);
}

fn run_prompt(interpreter: &mut Interpreter) {
    println!("Enter jlox code below...");

    // TODO: add history stack cycling on up arrow key press
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => run(interpreter, &line),
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}

fn run(interpreter: &mut Interpreter, source: &str) {
    let tokens = scan_tokens(source);
    let mut parser = parser::Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(e) => e,
        Err(err) => {
            println!("Parsing stage failed with error:");
            println!("  {}", err);
            return;
        }
    };

    match interpreter.interpret(statements) {
        Err(err) => println!("{}", err),
        _ => {}
    }
}
