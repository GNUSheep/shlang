use crate::{compiler::errors::error_message, vm::value::Value};
use std::io::{self, BufRead};

use super::print::print;

pub fn input(args: Vec<Value>) -> Value {
    if args.len() > 1 {
        error_message("shlang/std/input.rs".to_string(), "RUNTIME ERROR", "Too much arguments for INPUT function".to_string());
        std::process::exit(1);
    }

    if args.len() == 1 {
        print(args);
    }

    let stdin = io::stdin();
    let mut input = stdin.lock();

    let mut buffer = String::new();
    match input.read_line(&mut buffer) {
        Ok(_) => {
            buffer = buffer.trim_end_matches('\n').to_string();
        },
        Err(_) => {
            error_message("shlang/std/input.rs".to_string(), "INPUT ERROR", "Failed to get input".to_string());
            std::process::exit(1);
        },
    }

    Value::String(buffer)
}
