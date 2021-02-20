mod vm;

use vm::bytecode::{Chunk, OpCode, Value};
use vm::disassembler::disassemble_chunk;

fn main() {
    println!("Hello, world!");
    let mut chunk = Chunk::default();
    let offset = chunk.add_constant(Value::Number(1.2));
    chunk.write_chunk(OpCode::Constant(offset));

    chunk.write_chunk(OpCode::Return);
    print!("{}", disassemble_chunk(&chunk, "test chunk"));
}
