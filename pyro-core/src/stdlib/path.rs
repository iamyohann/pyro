use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;

fn join(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument (list of paths)".to_string())));
    }
    
    // Expect list of strings
    let parts: Vec<String> = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
        
    let mut path = std::path::PathBuf::new();
    for part in parts {
        path.push(part);
    }
    
    Ok(Value::String(Rc::new(path.display().to_string())))
}

fn basename(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let path_str: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    let path = Path::new(&path_str);
    match path.file_name() {
        Some(name) => Ok(Value::String(Rc::new(name.to_string_lossy().to_string()))),
        None => Ok(Value::String(Rc::new("".to_string()))),
    }
}

fn dirname(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let path_str: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    let path = Path::new(&path_str);
    match path.parent() {
        Some(name) => Ok(Value::String(Rc::new(name.to_string_lossy().to_string()))),
        None => Ok(Value::String(Rc::new("".to_string()))),
    }
}

fn extname(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let path_str: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    let path = Path::new(&path_str);
    match path.extension() {
        Some(name) => Ok(Value::String(Rc::new(name.to_string_lossy().to_string()))),
        None => Ok(Value::String(Rc::new("".to_string()))),
    }
}

fn abs_path(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let path_str: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    match std::fs::canonicalize(path_str) {
        Ok(path) => Ok(Value::String(Rc::new(path.display().to_string()))),
        Err(e) => Err(Value::String(Rc::new(e.to_string()))),
    }
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("join".to_string(), Value::NativeFunction {
        name: "join".to_string(),
        func: NativeClosure(Rc::new(join)),
    });
    methods.insert("basename".to_string(), Value::NativeFunction {
        name: "basename".to_string(),
        func: NativeClosure(Rc::new(basename)),
    });
    methods.insert("dirname".to_string(), Value::NativeFunction {
        name: "dirname".to_string(),
        func: NativeClosure(Rc::new(dirname)),
    });
    methods.insert("extname".to_string(), Value::NativeFunction {
        name: "extname".to_string(),
        func: NativeClosure(Rc::new(extname)),
    });
    methods.insert("abs_path".to_string(), Value::NativeFunction {
        name: "abs_path".to_string(),
        func: NativeClosure(Rc::new(abs_path)),
    });

    Value::NativeModule(Rc::new(methods))
}
