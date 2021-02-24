use std::{collections::HashMap, convert::TryFrom};
use string_interner::{Symbol};

use super::{bytecode::{ByteCode, Chunk, OpCode}, disassembler::disassemble_instruction, value::Value};

// FIXME: improve these messages to support line numbers.
#[derive(Debug)]
pub enum VmError {
  EmptyStack,
  TypeError(String),
  InvalidVariable(Value),  // bad interning
  UndefinedVariable,
  RuntimeError,
}

pub struct Vm<'a> {
  chunk: &'a mut Chunk,
  globals: HashMap<usize, Value>,
  ip: usize,
  stack: Vec<Value>,
}

static TRACE_VM: bool = false;

impl<'a> Vm<'a> {
  pub fn new(chunk: &'a mut Chunk) -> Vm<'a> {
    Vm {
      chunk,
      globals: HashMap::new(),
      ip: 0,
      stack: Vec::new(),
    }
  }

  pub fn run(&mut self) -> Result<(), VmError> {
    loop {
      let instr = self.read_byte().ok_or(VmError::RuntimeError)?;

      if TRACE_VM {
        let mut output = String::new();
        self.dump_stack(&mut output);
        disassemble_instruction(&self.chunk, instr, self.ip - 1, &mut output);
        println!("{}", output.as_str());
      }

      let opcode = OpCode::try_from(instr).or(Result::Err(VmError::RuntimeError))?;

      match opcode {
        OpCode::Constant => {
          let constant = self.read_constant().ok_or(VmError::RuntimeError)?.clone();
          self.stack.push(constant);
        },
        OpCode::Nil => self.stack.push(Value::Nil),
        OpCode::True => self.stack.push(Value::Bool(true)),
        OpCode::False => self.stack.push(Value::Bool(false)),
        OpCode::Pop => {
          self.stack.pop().ok_or(VmError::EmptyStack)?;
        },
        OpCode::GetGlobal => {
          let constant_idx = self.read_byte().ok_or(VmError::RuntimeError)?;
          let name = self.chunk.get_constant(constant_idx).ok_or(VmError::RuntimeError)?;
          let value = Vm::load(&mut self.globals, name)?;
          self.stack.push(value);
        },
        OpCode::DefineGlobal => {
          let constant_idx = self.read_byte().ok_or(VmError::RuntimeError)?;
          let name = self.chunk.get_constant(constant_idx).ok_or(VmError::RuntimeError)?;
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          Vm::store(&mut self.globals, name, value)?;
        },
        OpCode::Equal => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          self.stack.push(Value::Bool(a.equal(&b)));
        },
        OpCode::Greater => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.greater(&b)?;
          self.stack.push(Value::Bool(result));
        },
        OpCode::Less => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.less(&b)?;
          self.stack.push(Value::Bool(result));
        },
        OpCode::Add => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.add(&b, self.chunk.interner())?;
          self.stack.push(result);
        },
         OpCode::Subtract => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.subtract(&b)?;
          self.stack.push(result);
        },
        OpCode::Multiply => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.multiply(&b)?;
          self.stack.push(result);
        },
        OpCode::Divide => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let result = a.divide(&b)?;
          self.stack.push(result);
        },
        OpCode::Not => {
          let b = self.stack.pop().ok_or(VmError::EmptyStack)?;
          self.stack.push(Value::Bool(b.is_falsey(self.chunk.interner())));
        },
        OpCode::Negate => {
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let negated = value.negate()?;
          self.stack.push(negated);
        },
        OpCode::Print => {
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          println!("{}", value.to_string(self.chunk.interner()));
        },
        OpCode::Return => {
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          println!("{:?}", value);
          return Result::Ok(());
        },
      }
    }
  }

  fn load(map: &mut HashMap<usize, Value>, key: &Value) -> Result<Value, VmError> {
    match key {
      Value::InternedString(interned_key) => {
        match map.get(&interned_key.to_usize()) {
          // FIXME: avoid cloning values here.
          Some(val) => Ok(val.clone()),
          // FIXME: include actual string value here.
          None => Err(VmError::UndefinedVariable),
        }
      },
      _ => Err(VmError::InvalidVariable(key.clone())),
    }
  }

  fn store(map: &mut HashMap<usize, Value>, key: &Value,
      value: Value) -> Result<(), VmError> {
    match key {
      Value::InternedString(interned_key) => {
        map.insert(interned_key.to_usize(), value);
        Ok(())
      },
      _ => Err(VmError::InvalidVariable(key.clone())),
    }
  }

  fn read_byte(&mut self) -> Option<ByteCode> {
    let index = self.ip;
    self.ip += 1;
    return self.chunk.get_bytecode(index).copied();
  }

  fn read_constant(&mut self) -> Option<&Value> {
    let constant_idx = self.read_byte()?;
    return self.chunk.get_constant(constant_idx);
  }

  fn dump_stack(&self, output: &mut String) {
    output.push_str("          ");
    for value in &self.stack {
      output.push_str(format!("[{:?}]", value).as_str());
    }
    output.push_str("\n");
  }
}
