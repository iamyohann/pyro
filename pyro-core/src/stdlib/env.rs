use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::env;

fn var(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let key: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    match env::var(key) {
        Ok(val) => Ok(Value::String(Rc::new(val))),
        Err(_) => Ok(Value::Void), // Or error? Python returns None or raises. Rust returns Result. Let's return Void for missing.
    }
}

fn vars(_args: Vec<Value>) -> Result<Value, Value> {
    let mut map = Vec::new();
    for (k, v) in env::vars() {
        map.push((Value::String(Rc::new(k)), Value::String(Rc::new(v))));
    }
    Ok(Value::Dict(Rc::new(map)))
}

fn args(_args: Vec<Value>) -> Result<Value, Value> {
    let mut list = Vec::new();
    for arg in env::args() {
        list.push(Value::String(Rc::new(arg)));
    }
    Ok(Value::List(Rc::new(list)))
}

fn cwd(_args: Vec<Value>) -> Result<Value, Value> {
    match env::current_dir() {
        Ok(path) => Ok(Value::String(Rc::new(path.display().to_string()))),
        Err(e) => Err(Value::String(Rc::new(e.to_string()))),
    }
}

fn set_cwd(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }
    let path: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    
    match env::set_current_dir(path) {
        Ok(_) => Ok(Value::Void),
        Err(e) => Err(Value::String(Rc::new(e.to_string()))),
    }
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("var".to_string(), Value::NativeFunction {
        name: "var".to_string(),
        func: NativeClosure(Rc::new(var)),
    });
    methods.insert("vars".to_string(), Value::NativeFunction {
        name: "vars".to_string(),
        func: NativeClosure(Rc::new(vars)),
    });
    methods.insert("args".to_string(), Value::NativeFunction {
        name: "args".to_string(),
        func: NativeClosure(Rc::new(args)),
    });
    methods.insert("cwd".to_string(), Value::NativeFunction {
        name: "cwd".to_string(),
        func: NativeClosure(Rc::new(cwd)),
    });
    methods.insert("set_cwd".to_string(), Value::NativeFunction {
        name: "set_cwd".to_string(),
        func: NativeClosure(Rc::new(set_cwd)),
    });

    Value::NativeModule(Rc::new(methods))
}
