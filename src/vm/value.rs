#[derive(Debug, Copy, Clone)]
pub enum Value {
    Float(f64),
    Int(i64),
}

impl Value {
    pub fn get_float(&self) -> f64 {
        match self {
            Value::Float(val) => return *val,
            _ => {
                println!("ERROR: Can't get float value");
                std::process::exit(1);
            },
        }
    }
}

#[derive(Debug)]
pub struct ValuesArray {
    values: Vec<Value>,
}

impl ValuesArray {
    pub fn init() -> Self {
        Self {
            values: vec![],
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn get(&self, index: usize) -> Value {
        self.values[index]
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
