use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue, ToPyroValue};
use std::collections::HashMap;
use std::rc::Rc;

fn sqrt(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.sqrt().to_value())
}

fn abs(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.abs().to_value())
}

fn ceil(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.ceil().to_value())
}

fn floor(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.floor().to_value())
}

fn round(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.round().to_value())
}

fn sin(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.sin().to_value())
}

fn cos(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.cos().to_value())
}

fn tan(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Rc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(val.tan().to_value())
}

fn pow(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Rc::new("Expected 2 arguments".to_string())));
    }

    let base: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Rc::new(e)))?;
    let exp: f64 = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Rc::new(e)))?;

    Ok(base.powf(exp).to_value())
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("sqrt".to_string(), Value::NativeFunction {
        name: "sqrt".to_string(),
        func: NativeClosure(Rc::new(sqrt)),
    });
    methods.insert("abs".to_string(), Value::NativeFunction {
        name: "abs".to_string(),
        func: NativeClosure(Rc::new(abs)),
    });
    methods.insert("ceil".to_string(), Value::NativeFunction {
        name: "ceil".to_string(),
        func: NativeClosure(Rc::new(ceil)),
    });
    methods.insert("floor".to_string(), Value::NativeFunction {
        name: "floor".to_string(),
        func: NativeClosure(Rc::new(floor)),
    });
    methods.insert("round".to_string(), Value::NativeFunction {
        name: "round".to_string(),
        func: NativeClosure(Rc::new(round)),
    });
    methods.insert("sin".to_string(), Value::NativeFunction {
        name: "sin".to_string(),
        func: NativeClosure(Rc::new(sin)),
    });
    methods.insert("cos".to_string(), Value::NativeFunction {
        name: "cos".to_string(),
        func: NativeClosure(Rc::new(cos)),
    });
    methods.insert("tan".to_string(), Value::NativeFunction {
        name: "tan".to_string(),
        func: NativeClosure(Rc::new(tan)),
    });
    methods.insert("pow".to_string(), Value::NativeFunction {
        name: "pow".to_string(),
        func: NativeClosure(Rc::new(pow)),
    });

    Value::NativeModule(Rc::new(methods))
}
