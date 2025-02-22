use crate::vm::value::{self, Value};

pub trait Object {
    fn inc_counter(&mut self);
    fn dec_counter(&mut self);
    fn get_rc_counter(&self) -> usize;

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

    pub fn inc_counter(&mut self, index: usize) {
        self.get_object(index).inc_counter();
    }

    pub fn dec_counter(&mut self, index: usize) {
        let obj = self.get_object(index);
       
        obj.dec_counter();
    }

    fn dec_values(&mut self, index: usize) {
        for field_obj in self.get_object(index).get_values() {
            match field_obj {
                Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => {
                    self.get_object(heap_pos).dec_counter();
                }
                _ => {},
            }
        }
    }

    pub fn remove(&mut self) {
        for i in (0..self.heap.len()).rev() {
            if self.get_object(i).get_rc_counter() == 0 {
                self.dec_values(i);
                self.heap[i] = Box::new(EmptyObject{});
            }
        }

        // for i in 0..self.heap.len() {
        //     println!("{:?} RC: {:?}", self.get_object(i).get_values(), self.get_object(i).get_rc_counter());
        // }
    }

    pub fn remove_all(&mut self) {
        self.heap = vec![];
    }
}

pub struct EmptyObject {}

impl Object for EmptyObject {
    fn inc_counter(&mut self) {}
    
    fn dec_counter(&mut self) {}

    fn get_rc_counter(&self) -> usize {
        0
    }

    fn get_values(&self) -> Vec<Value> {
        vec![Value::EmptyObj]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {}

    fn get_arg_count(&self) -> usize {
        0
    }
}
