use crate::vm::value::{self, Value};

pub trait Object {
    fn inc_counter(&mut self);
    fn dec_counter(&mut self);
    fn get_rc_counter(&self) -> usize;

    fn set_index(&mut self, index: usize);
    fn get_index(&self) -> usize;

    fn get_values(&self) -> Vec<value::Value>;
    fn set_value(&mut self, pos: usize, value: value::Value); 
    fn get_arg_count(&self) -> usize;
}

pub struct ReferenceCounter {
    pub heap: Vec<Box<dyn Object>>,
}

impl ReferenceCounter {
    pub fn init() -> Self {
        Self {
            heap: vec![],
        }
    }

    pub fn push(&mut self, object: Box<dyn Object>) {
        self.heap.push(object);
    }

    pub fn get_object(&mut self, index: usize) -> &mut Box<dyn Object> {
        &mut self.heap[index]
    }

    pub fn find_object(&mut self, index: usize) -> usize {
        for i in 0..self.heap.len() {
            if self.heap[i].get_index() == index {
                return i;
            }
        }
        panic!();
    }

    pub fn inc_counter(&mut self, index: usize) {
        self.get_object(index).inc_counter();
    }

    pub fn dec_counter(&mut self, index: usize) {
        let obj = self.get_object(index);
       
        obj.dec_counter();
    }

    pub fn remove(&mut self) {
        for i in (0..self.heap.len()).rev() {
            if self.get_object(i).get_rc_counter() == 0 {
                self.heap.remove(i);
            }
        }
    }

    pub fn remove_all(&mut self) {
        self.heap = vec![];
    }
}

pub struct RefObject {
    pub ref_index: usize,
    pub rc_counter: usize,
    pub index: usize,
}

impl Object for RefObject {
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

    fn get_values(&self) -> Vec<Value> {
        vec![Value::InstanceRef(self.ref_index)]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {
    }

    fn get_arg_count(&self) -> usize {
        0
    }
}
