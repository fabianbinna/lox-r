mod scanner;
mod error;
mod token;
mod object;
mod function;
mod class;
mod instance;
mod expr;
mod stmt;
mod parser;
mod environment;
mod resolver;
mod interpreter;
mod native;

use std::{collections::HashMap, env, fs::read_to_string, io::{self, Write}, path::PathBuf, process::exit};
use interpreter::Interpreter;
use parser::parse;
use resolver::resolve;
use scanner::scan_tokens;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: lox-rs <script>");
        exit(64);
    } else if args.len() == 2 {
        run_file(args[1].clone());
    } else {
        run_prompt();
    }
}

fn run_prompt() {
    let mut interpreter = Interpreter::new(HashMap::new());
    loop {
        let mut input = String::new();
        print!("> ");
        let _ = io::stdout().flush();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let tokens = scan_tokens(input);
                let statements = parse(tokens);
                let locals = resolve(&statements);
                interpreter.extend_locals(locals);
                interpreter.interpret(&statements);
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                return;
            }
        }
    }
}

fn run_file(script_path: String) {
    let script_path_buf = PathBuf::from(script_path);
    let source = read_to_string(script_path_buf).expect("Could not read script.");
    run(source);
}

fn run(source: String) {  
    let tokens = scan_tokens(source);
    let statements = parse(tokens);
    let locals = resolve(&statements);
    let mut interpreter = Interpreter::new(locals);
    interpreter.interpret(&statements);
}