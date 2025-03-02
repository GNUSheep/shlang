use crate::{compiler::errors::error_message, vm::value::Value};


pub fn abs_int(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "ABS only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use ABS on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            return Value::Int(val.abs());
        },
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ABSINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}

pub fn abs_float(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "ABS only takes one argument".to_string());
        std::process::exit(1);
    }

    match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use ABS on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            return Value::Float(val.abs());
        },
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ABSFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    }
}

pub fn pow_int(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "POW only takes two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use POW on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("POWINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use POW on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("POWINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Int(a.pow(b as u32))
}

pub fn pow_float(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "POW only takes two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use POW on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("POWFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use POW on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("POWFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Float(a.powf(b))
}

pub fn min_int(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "MIN takes only two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MIN on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MININT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MIN on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MININT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Int(std::cmp::min(a, b))
}

pub fn min_float(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "MIN takes only two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MIN on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MININT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MIN on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MINFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Float(f64::min(a, b))
}

pub fn max_int(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "MAX takes only two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MAX on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MAXINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MAX on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MAXINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Int(std::cmp::max(a, b))
}

pub fn max_float(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "MAX takes only two arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MAX on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MAXFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let b = match args[1].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use MAX on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("MAXFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Float(f64::max(a, b))
}

pub fn sqrt_int(args: Vec<Value>) -> Value {
    if args.len() != 1{
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "SQRT takes only one arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use SQRT on string type"));
            std::process::exit(1);
        },
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("SQRTINT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Int((a as f64).sqrt() as i64)
}

pub fn sqrt_float(args: Vec<Value>) -> Value {
    if args.len() != 1{
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "SQRT takes only one arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use SQRT on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("SQRTFLOAT not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    Value::Float((a as f64).sqrt())
}

pub fn round(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "ROUND takes only one arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use ROUND on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let to = match args[1].clone() {
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND: point to where number should be rounded can by only INT TYPE, not: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let scale_factor = 10f64.powi(to as i32);
    Value::Float((a * scale_factor).round() / scale_factor) 
}

pub fn floor(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "ROUND takes only one arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use ROUND on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let to = match args[1].clone() {
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND: point to where number should be rounded can by only INT TYPE, not: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let scale_factor = 10f64.powi(to as i32);
    Value::Float((a * scale_factor).floor() / scale_factor) 
}

pub fn ceil(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", "ROUND takes only one arguments".to_string());
        std::process::exit(1);
    }

    let a = match args[0].clone() {
        Value::String(_) => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("Cannot use ROUND on string type"));
            std::process::exit(1);
        },
        Value::Float(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND not implemnted for this type: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let to = match args[1].clone() {
        Value::Int(val) => {
            val
        }
        _ => {
            error_message("shlang/std/math.rs".to_string(), "RUNTIME ERROR", format!("ROUND: point to where number should be rounded can by only INT TYPE, not: \"{:?}\"", args[0]));
            std::process::exit(1);
        }
    };

    let scale_factor = 10f64.powi(to as i32);
    Value::Float((a * scale_factor).ceil() / scale_factor) 
}
