use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;

use crate::interpreter::Interpreter;
use crate::scanner::{TokenType, scan_tokens};

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

    let (mut parser, _) = create_parser(&contents);
    run_stmts(interpreter, &mut parser);
}

fn run_prompt(interpreter: &mut Interpreter) {
    println!("Lox AST walk interpreter started.");

    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();

                let (mut parser, has_semicolon) = create_parser(&line);

                if has_semicolon {
                    run_stmts(interpreter, &mut parser);
                } else {
                    // assume if line to interpret does NOT have a ";" it was passed an expression
                    run_expr(interpreter, &mut parser);
                }
            }
            Err(ReadlineError::Interrupted) => break, // Ctrl-C
            Err(ReadlineError::Eof) => break,         // Ctrl-D
            Err(err) => println!("Error: {:?}", err),
        }
    }
}

fn run_stmts(interpreter: &mut Interpreter, parser: &mut parser::Parser) {
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

fn run_expr(interpreter: &mut Interpreter, parser: &mut parser::Parser) {
    let expr = match parser.expression() {
        Ok(e) => e,
        Err(err) => {
            println!("Parsing stage failed with error:");
            println!("  {}", err);
            return;
        }
    };

    match interpreter.evaluate_expr(&expr) {
        Ok(literal) => println!("{}", literal),
        Err(err) => println!("{}", err),
    }
}

fn create_parser(source: &str) -> (parser::Parser, bool) {
    let tokens = scan_tokens(source);

    let mut has_semicolon = false;
    for token in &tokens {
        if token.token_type == TokenType::SemiColon {
            has_semicolon = true;
        }
    }

    return (parser::Parser::new(tokens), has_semicolon);
}
