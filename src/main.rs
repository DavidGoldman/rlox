mod vm;

use vm::bytecode::{Chunk, OpCode};
use vm::value::Value;
use vm::vm::Vm;

fn main() {
    let mut chunk = Chunk::default();
    let mut constant = chunk.add_constant(Value::Number(1.2));
    chunk.write_op(OpCode::Constant, 123);
    chunk.write(constant, 123);

    constant = chunk.add_constant(Value::Number(3.4));
    chunk.write_op(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write_op(OpCode::Add, 123);

    constant = chunk.add_constant(Value::Number(5.6));
    chunk.write_op(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write_op(OpCode::Divide, 123);

    chunk.write_op(OpCode::Negate, 123);
    chunk.write_op(OpCode::Return, 123);

    let mut vm = Vm::new(&chunk);
    let res = vm.run();
    println!("{:?}", res);
}
