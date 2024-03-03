#[derive(Debug)]
pub enum OpCode {
    RETURN,
}

#[derive(Debug)]
pub struct Instruction {
    pub op: OpCode,
    pub bytes: u32,
}

pub struct Chunk { 
    code: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.code.push(instruction);
    }

    pub fn get_instruction(&self, offset: usize) -> &Instruction {
        &self.code[offset]
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
} 