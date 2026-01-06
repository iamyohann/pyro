use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::process::{Command, Stdio};

fn exit(args: Vec<Value>) -> Result<Value, Value> {
    let code = if args.len() > 0 {
        match &args[0] {
            Value::Int(i) => *i as i32,
            _ => 0,
        }
    } else {
        0
    };
    std::process::exit(code);
}

fn exec(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 1 {
        return Err(Value::String(Arc::new("Expected at least 1 argument (command)".to_string())));
    }
    
    let cmd_str: String = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
        
    let cmd_args: Vec<String> = if args.len() > 1 {
        FromPyroValue::from_value(&args[1]).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };
    
    let output = Command::new(cmd_str)
        .args(cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Value::String(Arc::new(e.to_string())))?;
        
    let mut result_map = Vec::new();
    result_map.push((
        Value::String(Arc::new("stdout".to_string())),
        Value::String(Arc::new(String::from_utf8_lossy(&output.stdout).to_string()))
    ));
    result_map.push((
        Value::String(Arc::new("stderr".to_string())),
        Value::String(Arc::new(String::from_utf8_lossy(&output.stderr).to_string()))
    ));
    result_map.push((
        Value::String(Arc::new("code".to_string())),
        Value::Int(output.status.code().unwrap_or(-1) as i64)
    ));
    
    Ok(Value::Dict(Arc::new(result_map)))
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("exit".to_string(), Value::NativeFunction {
        name: "exit".to_string(),
        func: NativeClosure(Arc::new(exit)),
    });
    methods.insert("exec".to_string(), Value::NativeFunction {
        name: "exec".to_string(),
        func: NativeClosure(Arc::new(exec)),
    });

    Value::NativeModule(Arc::new(methods))
}
