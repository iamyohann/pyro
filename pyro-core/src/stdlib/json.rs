use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue};
use std::collections::HashMap;
use std::sync::Arc;

use serde_json;

fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::String(s) => serde_json::Value::String(s.to_string()),
        Value::List(l) => {
            let vec: Vec<serde_json::Value> = l.iter().map(value_to_json).collect();
            serde_json::Value::Array(vec)
        },
        Value::ListMutable(l) => {
            let vec: Vec<serde_json::Value> = l.read().unwrap().iter().map(value_to_json).collect();
            serde_json::Value::Array(vec)
        },
        Value::Dict(d) => {
            let mut map = serde_json::Map::new();
            for (k, v) in d.iter() {
                if let Value::String(s) = k {
                    map.insert(s.to_string(), value_to_json(v));
                } else {
                     // serde_json keys must be strings
                }
            }
            serde_json::Value::Object(map)
        },
        Value::DictMutable(d) => {
            let mut map = serde_json::Map::new();
            for (k, v) in d.read().unwrap().iter() {
                if let Value::String(s) = k {
                    map.insert(s.to_string(), value_to_json(v));
                }
            }
            serde_json::Value::Object(map)
        },
        Value::Void => serde_json::Value::Null,
        // Tuples to arrays
        Value::Tuple(t) => {
             let vec: Vec<serde_json::Value> = t.iter().map(value_to_json).collect();
             serde_json::Value::Array(vec)
        },
        Value::TupleMutable(t) => {
             let vec: Vec<serde_json::Value> = t.read().unwrap().iter().map(value_to_json).collect();
             serde_json::Value::Array(vec)
        },
        // Sets to arrays
        Value::Set(s) => {
             let vec: Vec<serde_json::Value> = s.iter().map(value_to_json).collect();
             serde_json::Value::Array(vec)
        },
        Value::SetMutable(s) => {
             let vec: Vec<serde_json::Value> = s.read().unwrap().iter().map(value_to_json).collect();
             serde_json::Value::Array(vec)
        },
        _ => serde_json::Value::Null, 
    }
}

fn json_to_value(val: &serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Void,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Value::Int(n.as_i64().unwrap())
            } else if n.is_f64() {
                Value::Float(n.as_f64().unwrap())
            } else {
                Value::Int(n.as_i64().unwrap_or(0)) // fallback
            }
        },
        serde_json::Value::String(s) => Value::String(Arc::new(s.clone())),
        serde_json::Value::Array(arr) => {
            let vec: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::List(Arc::new(vec))
        },
        serde_json::Value::Object(obj) => {
            let mut vec = Vec::new();
            for (k, v) in obj {
                vec.push((Value::String(Arc::new(k.clone())), json_to_value(v)));
            }
            Value::Dict(Arc::new(vec))
        },
    }
}

fn stringify(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let json_val = value_to_json(&args[0]);
    match serde_json::to_string(&json_val) {
        Ok(s) => Ok(Value::String(Arc::new(s))),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn parse(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let s: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    
    match serde_json::from_str(&s) {
        Ok(v) => Ok(json_to_value(&v)),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("stringify".to_string(), Value::NativeFunction {
        name: "stringify".to_string(),
        func: NativeClosure(Arc::new(stringify)),
    });
    methods.insert("parse".to_string(), Value::NativeFunction {
        name: "parse".to_string(),
        func: NativeClosure(Arc::new(parse)),
    });

    Value::NativeModule(Arc::new(methods))
}
