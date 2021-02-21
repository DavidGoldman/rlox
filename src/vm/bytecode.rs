use std::{convert::TryInto, ops::Index, usize};

pub type Offset = usize;

pub type ByteCode = u8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
  Constant,
  Return,
}

#[derive(Debug, Clone)]
pub enum Value {
  Number(f64),
}

#[derive(Debug, Default)]
pub struct Chunk {
  code: Vec<ByteCode>,
  constants: Vec<Value>,
}

impl Chunk {
  pub fn write(&mut self, instr: ByteCode) {
    self.code.push(instr);
  }

  pub fn len(&self) -> usize {
    self.code.len()
  }

  pub fn add_constant(&mut self, value: Value) -> ByteCode {
    self.constants.push(value);
    // FIXME: This is unsafe, can panic at runtime if there's too many constants.
    (self.constants.len() - 1).try_into().unwrap()
  }

  pub fn get_constant(&self, offset: ByteCode) -> Option<&Value> {
    self.constants.get(offset as usize)
  }
}

impl Index<usize> for Chunk {
  type Output = ByteCode;
  fn index<'a>(&'a self, idx: usize) -> &'a ByteCode {
      &self.code[idx]
  }
}
