mod vm;

use vm::bytecode::{Chunk, OpCode, Value};
use vm::vm::Vm;

fn main() {
    let mut chunk = Chunk::default();
    let offset = chunk.add_constant(Value::Number(1.2));
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(offset, 123);
    chunk.write(OpCode::Return as u8, 123);

    let mut vm = Vm::new(&chunk);
    let res = vm.run();
    println!("{:?}", res);
}
