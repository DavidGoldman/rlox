#[derive(Debug, Clone)]
pub enum Value {
  Number(f64),
}

impl Value {
  pub fn add(&self, other: &Value) -> Option<Value> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Some(Number(a + b))
    }
  }

  pub fn subtract(&self, other: &Value) -> Option<Value> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Some(Number(a - b))
    }
  }

  pub fn multiply(&self, other: &Value) -> Option<Value> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Some(Number(a * b))
    }
  }

  pub fn divide(&self, other: &Value) -> Option<Value> {
    use Value::*;
    match (self, other) {
      (Number(a), Number(b)) => Some(Number(a / b))
    }
  }

  pub fn negate(&self) -> Option<Value> {
    use Value::*;
    match self {
      Number(number) => Some(Number(-number))
    }
  }
}
