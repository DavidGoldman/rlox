use std::{convert::TryFrom, ops::Index, usize};
use super::value::Value;

pub type Offset = usize;

/// `OpCode` or data.
pub type ByteCode = u8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
  Constant,
  Nil,
  True,
  False,
  Add,
  Subtract,
  Multiply,
  Divide,
  Not,
  Negate,
  Return,
}

impl TryFrom<ByteCode> for OpCode {
  type Error = ();

  // See https://stackoverflow.com/a/57578431, using this seems to have the same
  // performance for release builds as the manual switch we previously had in
  // multiple places.
  fn try_from(v: ByteCode) -> Result<Self, Self::Error> {
    use OpCode::*;
    match v {
      x if x == Constant as ByteCode => Ok(Constant),
      x if x == Nil as ByteCode => Ok(Nil),
      x if x == True as ByteCode => Ok(True),
      x if x == False as ByteCode => Ok(False),
      x if x == Add as ByteCode => Ok(Add),
      x if x == Subtract as ByteCode => Ok(Subtract),
      x if x == Multiply as ByteCode => Ok(Multiply),
      x if x == Divide as ByteCode => Ok(Divide),
      x if x == Not as ByteCode => Ok(Not),
      x if x == Negate as ByteCode => Ok(Negate),
      x if x == Return as ByteCode => Ok(Return),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Default)]
pub struct Chunk {
  code: Vec<ByteCode>,
  // FIXME: this representation is wasteful, see Chapter 14, challenge 1.
  lines: Vec<usize>,
  constants: Vec<Value>,
}

impl Chunk {
  pub fn write_op(&mut self, op: OpCode, line: usize) {
    self.write(op as u8, line);
  }

  pub fn write(&mut self, instr: ByteCode, line: usize) {
    self.code.push(instr);
    self.lines.push(line);
  }

  pub fn len(&self) -> usize {
    self.code.len()
  }

  pub fn add_constant(&mut self, value: Value) -> Result<ByteCode, Value> {
    let constant_idx = self.constants.len();
    if let Ok(bytecode_idx) = ByteCode::try_from(constant_idx) {
      self.constants.push(value);
      Ok(bytecode_idx)
    } else {
      Err(value)
    }
  }

  pub fn get_constant(&self, offset: ByteCode) -> Option<&Value> {
    self.constants.get(offset as usize)
  }

  pub fn get_bytecode(&self, offset: usize) -> Option<&ByteCode> {
    self.code.get(offset)
  }

  pub fn get_line(&self, offset: usize) -> usize {
    *self.lines.get(offset).unwrap_or(&0)
  }
}

impl Index<usize> for Chunk {
  type Output = ByteCode;
  fn index<'a>(&'a self, idx: usize) -> &'a ByteCode {
      &self.code[idx]
  }
}
