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

  if offset > 0 && chunk.get_line(offset) == chunk.get_line(offset - 1) {
    output.push_str("   | ");
  } else {
    output.push_str(format!("{:4} ", chunk.get_line(offset)).as_str());
  }

  // Work around the type differences via the suggestion here:
  // https://stackoverflow.com/a/28029667
  match instr {
    instr if instr == OpCode::Constant as ByteCode => {
      output.push_str(constant_instruction("OP_CONSTANT", chunk, offset).as_str());
      return offset + 2;
    }
    instr if instr == OpCode::Negate as ByteCode => {
      output.push_str("OP_NEGATE\n");
      return offset + 1;
    }
    instr if instr == OpCode::Return as ByteCode => {
      output.push_str("OP_RETURN\n");
      return offset + 1;
    }
    _ => {
      output.push_str(format!("<unknown opcode {}>\n", instr).as_str());
      return offset + 1;
    }
  }

  fn constant_instruction(name: &str, chunk: &Chunk, offset: Offset) -> String {
    if let Some(constant_idx) = chunk.get_bytecode(offset + 1) {
      match chunk.get_constant(*constant_idx) {
        Some(val) => {
          format!("{:<16} {:4} {:?}\n", name, constant_idx, val)
        }
        None => {
          format!("{} <invalid constant offset {}>\n", name, constant_idx)
        }
      }
    } else {
      format!("{} <invalid bytecode offset {}>\n", name, offset + 1)
    }
  }
}
