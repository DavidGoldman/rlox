use super::vm::VmError;

#[derive(Debug, Clone)]
pub enum Value {
  Nil,
  Bool(bool),
  Number(f64),
  String(String),
}

impl Value {
  pub fn is_falsey(&self) -> bool {
    use Value::*;
    match self {
      Nil => true,
      Bool(val) => !val,
      String(val) => return val.len() == 0,
      _ => false,
    }
  }

  pub fn equal(&self, other: &Value) -> bool {
    use Value::*;
    match (self, other) {
      (Nil, Nil) => true,
      (Bool(a), Bool(b)) => a == b,
      (Number(a), Number(b)) => a == b,
      (String(a), String(b)) => a == b,
      _ => false,
    }
  }

  pub fn greater(&self, other: &Value) -> Result<bool, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(a > b),
      _ => Err(VmError::TypeError("> requires two numbers".to_string())),
    }
  }

  pub fn less(&self, other: &Value) -> Result<bool, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(a < b),
      _ => Err(VmError::TypeError("< requires two numbers".to_string())),
    }
  }

  pub fn add(&self, other: &Value) -> Result<Value, VmError> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Ok(Number(a + b)),
      (String(a), String(b)) => Ok(String(a.to_owned() + b)),
      _ => Err(VmError::TypeError("+ requires two numbers or strings".to_string())),
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
