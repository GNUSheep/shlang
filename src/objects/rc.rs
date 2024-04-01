use crate::vm::value;

pub trait Object {
    fn inc_counter(&mut self);
    fn dec_counter(&mut self);
    fn get_rc_counter(&self) -> usize;

    fn set_index(&mut self, index: usize);
    fn get_index(&self) -> usize;

    fn get_values(&self) -> Vec<value::Value>;
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
        
        if obj.get_rc_counter() == 0 {
            self.remove(index);
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.heap.remove(index);

        for (i, obj) in self.heap.iter_mut().enumerate() {
            obj.set_index(i);
        }
    }

    pub fn remove_all(&mut self) {
        self.heap = vec![];
    }
}