use crate::vm::value::Value;

pub fn println(args: Vec<Value>) -> Value {
    println!("NATIVE {:?}", args);
    Value::Null
}

pub fn test_returning(_args: Vec<Value>) -> Value {
    Value::Int(5)
}