use std::convert::TryFrom;

use super::{bytecode::{ByteCode, Chunk, OpCode}, disassembler::disassemble_instruction, value::Value};

// FIXME: improve these messages to support line numbers.
#[derive(Debug)]
pub enum VmError {
  EmptyStack,
  TypeError(String),
  CompileError,
  RuntimeError,
}

pub struct Vm<'a> {
  chunk: &'a Chunk,
  ip: usize,
  stack: Vec<Value>,
}

static TRACE_VM: bool = false;

impl<'a> Vm<'a> {
  pub fn new(chunk: &'a Chunk) -> Vm<'a> {
    Vm {
      chunk,
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
          let result = a.add(&b)?;
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
          self.stack.push(Value::Bool(b.is_falsey()));
        },
        OpCode::Negate => {
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          let negated = value.negate()?;
          self.stack.push(negated);
        },
        OpCode::Return => {
          let value = self.stack.pop().ok_or(VmError::EmptyStack)?;
          println!("{:?}", value);
          return Result::Ok(());
        },
      }
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
