use super::vm::VmError;

#[derive(Debug, Clone)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
}

impl Value {
  pub fn is_falsey(&self) -> bool {
    use Value::*;
    match self {
      Nil => true,
      Bool(val) => !val,
      _ => false,
    }
  }

  pub fn add(&self, other: &Value) -> Result<Value, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(Number(a + b)),
      _ => Err(VmError::TypeError("+ requires two numbers".to_string())),
    }
  }

  pub fn subtract(&self, other: &Value) -> Result<Value, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(Number(a - b)),
      _ => Err(VmError::TypeError("- requires two numbers".to_string())),
    }
  }

  pub fn multiply(&self, other: &Value) -> Result<Value, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(Number(a * b)),
      _ => Err(VmError::TypeError("* requires two numbers".to_string())),
    }
  }

  pub fn divide(&self, other: &Value) -> Result<Value, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(Number(a / b)),
      _ => Err(VmError::TypeError("/ requires two numbers".to_string())),
    }
  }

  pub fn negate(&self) -> Result<Value, VmError> {
    use Value::*;
    match self {
      Number(number) => Ok(Number(-number)),
      _ => Err(VmError::TypeError("- requires one number".to_string())),
    }
  }
}
