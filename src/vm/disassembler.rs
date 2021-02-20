use super::bytecode::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) -> String {
  let mut result = format!("== {} ==\n", name);
  for (idx, instr) in chunk.instructions().enumerate() {
    disassemble_instruction(*instr, idx, &mut result);
  }
  result
}

pub fn disassemble_instruction(instr: OpCode, offset: usize, output: &mut String) {
  output.push_str(format!("{:04} ", offset).as_str());

  match instr {
      OpCode::Return => {
        output.push_str("OP_RETURN\n");
      }
  }
}
