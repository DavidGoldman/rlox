use std::{convert::TryFrom, ops::Index, usize};
use string_interner::StringInterner;

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
  Pop,
  DefineGlobal,
  Equal,
  Greater,
  Less,
  Add,
  Subtract,
  Multiply,
  Divide,
  Not,
  Negate,
  Print,
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
      x if x == Pop as ByteCode => Ok(Pop),
      x if x == DefineGlobal as ByteCode => Ok(DefineGlobal),
      x if x == Equal as ByteCode => Ok(Equal),
      x if x == Greater as ByteCode => Ok(Greater),
      x if x == Less as ByteCode => Ok(Less),
      x if x == Add as ByteCode => Ok(Add),
      x if x == Subtract as ByteCode => Ok(Subtract),
      x if x == Multiply as ByteCode => Ok(Multiply),
      x if x == Divide as ByteCode => Ok(Divide),
      x if x == Not as ByteCode => Ok(Not),
      x if x == Negate as ByteCode => Ok(Negate),
      x if x == Print as ByteCode => Ok(Print),
      x if x == Return as ByteCode => Ok(Return),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Default)]
pub struct Chunk {
  code: Vec<ByteCode>,
  strings: StringInterner,
  // FIXME: this representation is wasteful, see Chapter 14, challenge 1.
  lines: Vec<usize>,
  constants: Vec<Value>,
}

pub(crate) enum ChunkConstant<'a> {
  Number(f64),
  String(&'a str),
}

impl Chunk {
  pub fn write(&mut self, instr: ByteCode, line: usize) {
    self.code.push(instr);
    self.lines.push(line);
  }

  pub fn len(&self) -> usize {
    self.code.len()
  }

  pub(crate) fn add_constant(&mut self, constant: ChunkConstant) -> Result<ByteCode, Value> {
    let constant_idx = self.constants.len();
    if let Ok(bytecode_idx) = ByteCode::try_from(constant_idx) {
      let value = self.value_for_constant(constant);
      self.constants.push(value);
      Ok(bytecode_idx)
    } else {
      Err(self.value_for_constant(constant))
    }
  }

  fn value_for_constant(&mut self, constant: ChunkConstant) -> Value {
    match constant {
      ChunkConstant::Number(num) => Value::Number(num),
      ChunkConstant::String(str) => Value::InternedString(self.strings.get_or_intern(str)),
    }
  }

  pub fn interner(&mut self) -> &mut StringInterner {
    &mut self.strings
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
