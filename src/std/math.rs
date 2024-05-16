use crate::{compiler::errors::error_message, vm::value::Value};


pub fn abs_int(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("RUNTIME ERROR", "ABS only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::String(_) => {
            error_message("RUNTIME ERROR", format!("Cannot use ABS on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            return Value::Int(val.abs());
        },
        _ => {
            error_message("RUNTIME ERROR", format!("ABSINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}

pub fn abs_float(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("RUNTIME ERROR", "ABS only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::String(_) => {
            error_message("RUNTIME ERROR", format!("Cannot use ABS on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            return Value::Float(val.abs());
        },
        _ => {
            error_message("RUNTIME ERROR", format!("ABSFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}