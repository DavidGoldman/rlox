pub type Offset = usize;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
  Constant(Offset),
  Return,
}

#[derive(Debug, Clone)]
pub enum Value {
  Number(f64),
}

#[derive(Debug, Default)]
pub struct Chunk {
  code: Vec<OpCode>,
  constants: Vec<Value>,
}

impl Chunk {
  pub fn write_chunk(&mut self, instr: OpCode) {
    self.code.push(instr);
  }

  pub fn add_constant(&mut self, value: Value) -> Offset {
    self.constants.push(value);
    self.constants.len() - 1
  }

  pub fn get_constant(&self, offset: Offset) -> Option<&Value> {
    self.constants.get(offset)
  }

  pub fn instructions(&self) -> std::slice::Iter<'_, OpCode> {
    self.code.iter()
  }
}
