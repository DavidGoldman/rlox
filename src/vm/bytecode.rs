use std::{convert::TryInto, ops::Index, usize};

pub type Offset = usize;

/// `OpCode` or data.
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
  // FIXME: this representation is wasteful, see Chapter 14, challenge 1.
  lines: Vec<u32>,
  constants: Vec<Value>,
}

impl Chunk {
  pub fn write(&mut self, instr: ByteCode, line: u32) {
    self.code.push(instr);
    self.lines.push(line);
  }

  pub fn len(&self) -> usize {
    self.code.len()
  }

  pub fn add_constant(&mut self, value: Value) -> ByteCode {
    let constant_idx = self.constants.len();
    self.constants.push(value);
    // FIXME: This is unsafe, can panic at runtime if there's too many constants.
    constant_idx.try_into().unwrap()
  }

  pub fn get_constant(&self, offset: ByteCode) -> Option<&Value> {
    self.constants.get(offset as usize)
  }

  pub fn get_bytecode(&self, offset: usize) -> Option<&ByteCode> {
    self.code.get(offset)
  }

  pub fn get_line(&self, offset: usize) -> u32 {
    *self.lines.get(offset).unwrap_or(&0)
  }
}

impl Index<usize> for Chunk {
  type Output = ByteCode;
  fn index<'a>(&'a self, idx: usize) -> &'a ByteCode {
      &self.code[idx]
  }
}
