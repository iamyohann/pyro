use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue, ToPyroValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::fs;

fn read_file(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    match fs::read_to_string(path) {
        Ok(content) => Ok(content.to_value()),
        Err(e) => Err(Value::String(Rc::new(e.to_string()))),
    }
}

fn write_file(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Rc::new("Expected 2 arguments".to_string())));
    }

    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    let content: String = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Rc::new(e)))?;

    match fs::write(path, content) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Rc::new(e.to_string()))),
    }
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("read_file".to_string(), Value::NativeFunction {
        name: "read_file".to_string(),
        func: NativeClosure(Rc::new(read_file)),
    });

    methods.insert("write_file".to_string(), Value::NativeFunction {
        name: "write_file".to_string(),
        func: NativeClosure(Rc::new(write_file)),
    });

    Value::NativeModule(Rc::new(methods))
}
