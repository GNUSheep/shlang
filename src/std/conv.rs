use crate::{compiler::errors::error_message, vm::value::Value, objects::string::StringMethods};

pub fn conv_to_int(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("RUNTIME ERROR", "CONV only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::String(val) => {
            if !StringMethods::is_digit(args).get_bool() {
                error_message("RUNTIME ERROR", format!("Cannot CONV this string, because it doesn't contains only digits"));
                std::process::exit(1);
            }

            return Value::Int(val.parse::<i64>().unwrap());
        }
        _ => {
            error_message("RUNTIME ERROR", format!("CONV not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}

pub fn conv_to_string(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("RUNTIME ERROR", "CONV only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::Int(val) => {
            return Value::String(val.to_string());
        }
        _ => {
            error_message("RUNTIME ERROR", format!("CONV not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}