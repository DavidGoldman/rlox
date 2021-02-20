mod vm;

use vm::bytecode::{Chunk, OpCode};
use vm::disassembler::disassemble_chunk;

fn main() {
    println!("Hello, world!");
    let mut chunk = Chunk::default();
    chunk.write_chunk(OpCode::Return);
    print!("{}", disassemble_chunk(&chunk, "test chunk"));
}
