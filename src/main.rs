mod vm;

use vm::bytecode::{Chunk, OpCode, Value};
use vm::disassembler::disassemble_chunk;

fn main() {
    let mut chunk = Chunk::default();
    let offset = chunk.add_constant(Value::Number(1.2));
    chunk.write(OpCode::Constant as u8);
    chunk.write(offset);
    chunk.write(OpCode::Return as u8);
    print!("{}", disassemble_chunk(&chunk, "test chunk"));
}
