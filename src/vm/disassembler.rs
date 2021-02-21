use std::convert::TryFrom;

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

  if let Ok(opcode) = OpCode::try_from(instr) {
    match opcode {
      OpCode::Constant => {
        output.push_str(constant_instruction("Constant", chunk, offset).as_str());
        return offset + 2;
      }
      val => {
        output.push_str(format!("{:?}\n", val).as_str());
        return offset + 1;
      }
    }
  } else {
    output.push_str(format!("<unknown opcode {}>\n", instr).as_str());
    return offset + 1;
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
