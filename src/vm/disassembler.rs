use super::bytecode::{ByteCode, Chunk, Offset, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) -> String {
  let mut result = format!("== {} ==\n", name);
  let len = chunk.len();
  let mut index: usize = 0;
  while index < len {
    index = disassemble_instruction(chunk, chunk[index], index, &mut result);
  }
  result
}

pub fn disassemble_instruction(
    chunk: &Chunk, instr: ByteCode, offset: usize, output: &mut String) -> usize {
  output.push_str(format!("{:04} ", offset).as_str());

  // Work around the type differences via the suggestion here:
  // https://stackoverflow.com/a/28029667
  match instr {
    instr if instr == OpCode::Return as ByteCode => {
      output.push_str("OP_RETURN\n");
      return offset + 1;
    }
    instr if instr == OpCode::Constant as ByteCode => {
      output.push_str(constant_instruction("OP_CONSTANT", chunk, offset).as_str());
      return offset + 2;
    }
    _ => {
      output.push_str(format!("<unknown opcode {}>\n", instr).as_str());
      return offset + 1;
    }
  }

  fn constant_instruction(name: &str, chunk: &Chunk, offset: Offset) -> String {
    // FIXME: This access is unsafe.
    let constant_idx = chunk[offset + 1];
    match chunk.get_constant(constant_idx) {
      Some(val) => {
        format!("{} {:?}\n", name, val)
      }
      None => {
        format!("{} <invalid offset {}>\n", name, constant_idx)
      }
    }
  }
}
