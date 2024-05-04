use crate::vm::value::Value;
use crate::compiler::errors;
use std::io::{self, Write};

pub fn println(args: Vec<Value>) -> Value {
    print(args);

    let stdout = io::stdout();
    let mut output = stdout.lock();

    match write!(output, "\n") {
        Ok(_) => {},
        Err(_) => {
            errors::error_message("PRINTING ERROR", format!("Failed to write newline to stdout"));
            std::process::exit(1);
        },
    };

    Value::Null
}

pub fn print(args: Vec<Value>) -> Value {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    for arg in args {
        match write!(output, "{}", arg) {
            Ok(_) => {},
            Err(_) => {
                errors::error_message("PRINTING ERROR", format!("Failed to write to stdout {}", arg));
                std::process::exit(1);
            },
        };
    }

    match output.flush() {
        Ok(_) => {},
        Err(_) => {
            errors::error_message("PRINTING ERROR", format!("Failed to flush stdout"));
            std::process::exit(1);
        },
    }

    Value::Null
}

pub fn test_returning(_args: Vec<Value>) -> Value {
    Value::Int(5)
}