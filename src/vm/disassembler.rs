use super::bytecode::{Chunk, Offset, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) -> String {
  let mut result = format!("== {} ==\n", name);
  for (idx, instr) in chunk.instructions().enumerate() {
    disassemble_instruction(chunk, *instr, idx, &mut result);
  }
  result
}

pub fn disassemble_instruction(
    chunk: &Chunk, instr: OpCode, offset: usize, output: &mut String) {
  output.push_str(format!("{:04} ", offset).as_str());

  match instr {
    OpCode::Return => {
      output.push_str("OP_RETURN\n");
    }
    OpCode::Constant(offset) => {
      output.push_str(constant_instruction("OP_CONSTANT", chunk, offset).as_str());
    }
  }

  fn constant_instruction(name: &str, chunk: &Chunk, offset: Offset) -> String {
    match chunk.get_constant(offset) {
      Some(val) => {
        format!("{} {:?}\n", name, val)
      }
      None => {
        format!("{} <invalid offset {}>\n", name, offset)
      }
    }
  }
}
