#[derive(Debug)]
pub enum Value {
    Float(f64),
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

    pub fn get(&self, index: usize) -> &Value {
        &self.values[index]
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

