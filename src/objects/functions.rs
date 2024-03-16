use crate::{
    objects::rc, vm::{bytecode, value::Value}
};

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub chunk: bytecode::Chunk,
    arg_count: u32,
    rc_counter: usize,
    index: usize,
}

impl rc::Object for Function {
    fn inc_counter(&mut self) {
        self.rc_counter += 1;
    }
    
    fn dec_counter(&mut self) {
        self.rc_counter -= 1;
    }

    fn get_rc_counter(&self) -> usize {
        self.rc_counter
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_value(&self) -> Value {
        Value::Chunk(self.chunk.clone())
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            chunk: bytecode::Chunk::new(),
            arg_count: 0,
            rc_counter: 1,
            index: 0,
        }
    }

    pub fn get_chunk(&mut self) -> &mut bytecode::Chunk {
        &mut self.chunk
    }
}