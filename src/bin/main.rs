use lox::scanner;
use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => println!("Usage: lox [filename]"),
    }
}

fn run_file(file: &String) {
    let src = fs::read_to_string(file).unwrap_or_else(|_| {
        println!("Failed to read file {}", file);
        String::new()
    });

    run(src);
}

fn run_prompt() {
    loop {
        print!("> ");
        std::io::stdout().flush().expect("print > to stdout");

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => run(line),
            Err(err) => {
                println!("Failed to read line: {}", err);
                return;
            }
        }
    }
}

fn run(src: String) {
    let tokens = scanner::scan_tokens(src).unwrap_or_else(|| Vec::new());
    for token in &tokens {
        println!("{}", token);
    }
}
