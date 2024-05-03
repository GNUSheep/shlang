use crate::{compiler::errors::error_message, vm::value::Value};
use crate::compiler::errors;
use std::io::{self, BufRead};

use super::print::print;

pub fn input(args: Vec<Value>) -> Value {
    if args.len() > 1 {
        error_message("RUNTIME ERROR", "Too much arguments for INPUT function".to_string());
        std::process::exit(1);
    }

    if args.len() == 1 {
        print(args);
    }

    Value::Null
}