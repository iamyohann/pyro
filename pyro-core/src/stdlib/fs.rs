use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue, ToPyroValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::fs;
use std::path::Path;

fn read_to_string(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    match fs::read_to_string(path) {
        Ok(content) => Ok(content.to_value()),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn write(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Arc::new("Expected 2 arguments".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    let content: String = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Arc::new(e)))?;
    match fs::write(path, content) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn exists(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(Value::Bool(Path::new(&path).exists()))
}

fn is_file(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(Value::Bool(Path::new(&path).is_file()))
}

fn is_dir(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(Value::Bool(Path::new(&path).is_dir()))
}

fn create_dir(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    match fs::create_dir_all(path) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn remove_file(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    match fs::remove_file(path) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn remove_dir(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    match fs::remove_dir(path) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

fn list_dir(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut result = Vec::new();
            for entry in entries {
                match entry {
                    Ok(e) => {
                        if let Ok(name) = e.file_name().into_string() {
                            result.push(Value::String(Arc::new(name)));
                        }
                    },
                    Err(_) => continue,
                }
            }
            Ok(Value::List(Arc::new(result)))
        },
        Err(e) => Err(Value::String(Arc::new(e.to_string()))),
    }
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("read_to_string".to_string(), Value::NativeFunction {
        name: "read_to_string".to_string(),
        func: NativeClosure(Arc::new(read_to_string)),
    });
    methods.insert("write".to_string(), Value::NativeFunction {
        name: "write".to_string(),
        func: NativeClosure(Arc::new(write)),
    });
    methods.insert("exists".to_string(), Value::NativeFunction {
        name: "exists".to_string(),
        func: NativeClosure(Arc::new(exists)),
    });
    methods.insert("is_file".to_string(), Value::NativeFunction {
        name: "is_file".to_string(),
        func: NativeClosure(Arc::new(is_file)),
    });
    methods.insert("is_dir".to_string(), Value::NativeFunction {
        name: "is_dir".to_string(),
        func: NativeClosure(Arc::new(is_dir)),
    });
    methods.insert("create_dir".to_string(), Value::NativeFunction {
        name: "create_dir".to_string(),
        func: NativeClosure(Arc::new(create_dir)),
    });
    methods.insert("remove_file".to_string(), Value::NativeFunction {
        name: "remove_file".to_string(),
        func: NativeClosure(Arc::new(remove_file)),
    });
    methods.insert("remove_dir".to_string(), Value::NativeFunction {
        name: "remove_dir".to_string(),
        func: NativeClosure(Arc::new(remove_dir)),
    });
    methods.insert("list_dir".to_string(), Value::NativeFunction {
        name: "list_dir".to_string(),
        func: NativeClosure(Arc::new(list_dir)),
    });

    Value::NativeModule(Arc::new(methods))
}
