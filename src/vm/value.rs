#[derive(Debug, Clone)]
pub enum Value {
  Number(f64),
}

impl Value {
  pub fn negate(&self) -> Option<Value> {
    match self {
        Value::Number(number) => Some(Value::Number(-number))
    }
  } 
}
