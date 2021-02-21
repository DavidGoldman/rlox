use super::{bytecode::{ByteCode, Chunk, OpCode, Value}, disassembler::disassemble_instruction};

#[derive(Debug)]
pub enum VmError {
  CompileError,
  RuntimeError,
}

pub struct Vm<'a> {
  chunk: &'a Chunk,
  ip: usize,
}

static TRACE_VM: bool = false;

impl<'a> Vm<'a> {
  pub fn new(chunk: &'a Chunk) -> Vm<'a> {
    Vm {
      chunk,
      ip: 0,
    }
  }

  pub fn run(&mut self) -> Result<(), VmError> {
    loop {
      let instr = *self.read_byte().ok_or(VmError::RuntimeError)?;

      if TRACE_VM {
        let mut output = String::new();
        disassemble_instruction(&self.chunk, instr, self.ip - 1, &mut output);
        println!("{}", output.as_str());
      }

      match instr {
        instr if instr == OpCode::Return as ByteCode => {
          return Result::Ok(());
        },
        instr if instr == OpCode::Constant as ByteCode => {
          let constant = self.read_constant().ok_or(VmError::RuntimeError)?;
          println!("{:?}", constant);
        },
        _ => {
          return Result::Err(VmError::RuntimeError);
        }
      }
    }
  }

  fn read_byte(&mut self) -> Option<&ByteCode> {
    let index = self.ip;
    self.ip += 1;
    return self.chunk.get_bytecode(index);
  }

  fn read_constant(&mut self) -> Option<&Value> {
    let constant_idx = *self.read_byte()?;
    return self.chunk.get_constant(constant_idx);
  }
}
