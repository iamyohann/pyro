use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue};
use std::collections::HashMap;
use std::rc::Rc;
use rand::Rng;

fn random(_args: Vec<Value>) -> Result<Value, Value> {
    Ok(Value::Float(rand::random()))
}

fn randint(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Rc::new("Expected 2 arguments (min, max)".to_string())));
    }
    let min: i64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    let max: i64 = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Rc::new(e)))?;
        
    let val = rand::thread_rng().gen_range(min..=max);
    Ok(Value::Int(val))
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("random".to_string(), Value::NativeFunction {
        name: "random".to_string(),
        func: NativeClosure(Rc::new(random)),
    });
    methods.insert("randint".to_string(), Value::NativeFunction {
        name: "randint".to_string(),
        func: NativeClosure(Rc::new(randint)),
    });

    Value::NativeModule(Rc::new(methods))
}
