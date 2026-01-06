use crate::interpreter::Value;
use std::sync::Arc;

pub trait FromPyroValue: Sized {
    fn from_value(v: &Value) -> Result<Self, String>;
}

pub trait ToPyroValue {
    fn to_value(self) -> Value;
}

impl FromPyroValue for i64 {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Int(i) => Ok(*i),
            _ => Err("Expected Int".to_string()),
        }
    }
}

impl ToPyroValue for i64 {
    fn to_value(self) -> Value {
        Value::Int(self)
    }
}

impl FromPyroValue for f64 {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            _ => Err("Expected Float or Int".to_string()),
        }
    }
}

impl ToPyroValue for f64 {
    fn to_value(self) -> Value {
        Value::Float(self)
    }
}

impl FromPyroValue for bool {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Bool(b) => Ok(*b),
            _ => Err("Expected Bool".to_string()),
        }
    }
}

impl ToPyroValue for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }
}

impl FromPyroValue for String {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::String(s) => Ok(s.to_string()),
            _ => Err("Expected String".to_string()),
        }
    }
}

impl ToPyroValue for String {
    fn to_value(self) -> Value {
        Value::String(Arc::new(self))
    }
}

// Implement for Vec<T>
impl<T: FromPyroValue> FromPyroValue for Vec<T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::List(l) => {
                let mut result = Vec::new();
                for item in l.iter() {
                    result.push(T::from_value(item)?);
                }
                Ok(result)
            }
            Value::ListMutable(l) => {
                let mut result = Vec::new();
                for item in l.read().unwrap().iter() {
                    result.push(T::from_value(item)?);
                }
                Ok(result)
            }
            _ => Err("Expected List".to_string()),
        }
    }
}

impl<T: ToPyroValue> ToPyroValue for Vec<T> {
    fn to_value(self) -> Value {
        let mut result = Vec::new();
        for item in self {
            result.push(item.to_value());
        }
        Value::List(Arc::new(result))
    }
}

impl ToPyroValue for () {
    fn to_value(self) -> Value {
        Value::Void
    }
}
