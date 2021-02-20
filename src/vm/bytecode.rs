#[derive(Debug, Clone, Copy)]
pub enum OpCode {
  Return,
}

#[derive(Debug, Default)]
pub struct Chunk {
  instructions: Vec<OpCode>,
}

impl Chunk {
  pub fn write_chunk(&mut self, instr: OpCode) {
    self.instructions.push(instr);
  }

  pub fn instructions(&self) -> std::slice::Iter<'_, OpCode> {
    self.instructions.iter()
  }
}
