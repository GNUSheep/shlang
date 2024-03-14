pub struct TestObject {
    pub name: String,
    pub value: i32,
    pub rc_counter: usize,
    pub index: usize,
}

impl Object for TestObject {
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
}

pub trait Object {
    fn inc_counter(&mut self);
    fn dec_counter(&mut self);
    fn get_rc_counter(&self) -> usize;

    fn set_index(&mut self, index: usize);
    fn get_index(&self) -> usize;
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
}