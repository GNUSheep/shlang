use crate::vm::bytecode;

pub fn debug_instruction(chunk: &bytecode::Chunk, offset: usize) -> usize {
    println!("{:#010x}", offset);

    let instruction = chunk.get_instruction(offset);
    match instruction.op {
            bytecode::OpCode::RETURN => {
                println!("{:?}", instruction);
            }
            bytecode::OpCode::CONSTANT_FLOAT(index) => {
                println!("{:?} => {:?}", instruction, chunk.get_value(index));
            }
            bytecode::OpCode::CONSTANT_INT(index) => {
                println!("{:?} => {:?}", instruction, chunk.get_value(index));
            }
            bytecode::OpCode::ADD_FLOAT | bytecode::OpCode::SUB_FLOAT | bytecode::OpCode::MUL_FLOAT | bytecode::OpCode::DIV_FLOAT => {
                println!("{:?}", instruction);
            }
            bytecode::OpCode::ADD_INT | bytecode::OpCode::SUB_INT | bytecode::OpCode::MUL_INT | bytecode::OpCode::DIV_INT => {
                println!("{:?}", instruction);
            }
    }

    offset + 1
}

pub fn debug_chunk(chunk: &bytecode::Chunk) {
    let op_len = chunk.len();
    let mut offset: usize = 0;

    while offset < op_len {
        offset = debug_instruction(chunk, offset);
    }
}