use clap::Parser;
use std::{
    fs,
    io::{self, BufRead},
};

use crate::scanner::scan_tokens;

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

    match args.script {
        Some(script) => run_file(&script),
        None => run_prompt(),
    };
}

fn run_file(script_path: &str) {
    let Ok(contents) = fs::read_to_string(script_path) else {
        panic!("Failed to find script");
    };

    run(&contents);
}

fn run_prompt() {
    println!("Enter jlox code below...");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => run(&line),
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}

fn run(source: &str) {
    let tokens = scan_tokens(source);

    for token in tokens {
        println!("{:?}", token);
    }
}
