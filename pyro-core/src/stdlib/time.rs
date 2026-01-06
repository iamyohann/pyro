use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue, ToPyroValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

fn now(_args: Vec<Value>) -> Result<Value, Value> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Value::String(Rc::new(e.to_string())))?;
    Ok(since_the_epoch.as_secs_f64().to_value())
}

fn sleep(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let seconds: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    thread::sleep(Duration::from_secs_f64(seconds));

    Ok(Value::Void)
}

fn millis(_args: Vec<Value>) -> Result<Value, Value> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Value::String(Rc::new(e.to_string())))?;
    Ok(Value::Int(since_the_epoch.as_millis() as i64))
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("now".to_string(), Value::NativeFunction {
        name: "now".to_string(),
        func: NativeClosure(Rc::new(now)),
    });

    methods.insert("sleep".to_string(), Value::NativeFunction {
        name: "sleep".to_string(),
        func: NativeClosure(Rc::new(sleep)),
    });
    methods.insert("millis".to_string(), Value::NativeFunction {
        name: "millis".to_string(),
        func: NativeClosure(Rc::new(millis)),
    });

    Value::NativeModule(Rc::new(methods))
}
