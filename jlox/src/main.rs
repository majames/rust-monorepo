use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to jlox script to execute
    #[arg(num_args = 0..=1)]
    script: Option<String>,
}

fn main() {
    Args::parse();
    println!("Hello, world!");
}
