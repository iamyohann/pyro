use crate::interpreter::{Value, NativeClosure};
use crate::convert::{FromPyroValue, ToPyroValue};
use std::collections::HashMap;
use std::sync::Arc;

fn sqrt(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.sqrt().to_value())
}

fn abs(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.abs().to_value())
}

fn ceil(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.ceil().to_value())
}

fn floor(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.floor().to_value())
}

fn round(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.round().to_value())
}

fn sin(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.sin().to_value())
}

fn cos(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.cos().to_value())
}

fn tan(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }

    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(val.tan().to_value())
}

fn pow(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Arc::new("Expected 2 arguments".to_string())));
    }

    let base: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    let exp: f64 = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Arc::new(e)))?;

    Ok(base.powf(exp).to_value())
}

fn asin(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.asin().to_value())
}

fn acos(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.acos().to_value())
}

fn atan(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.atan().to_value())
}

fn atan2(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Arc::new("Expected 2 arguments".to_string())));
    }
    let y: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    let x: f64 = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(y.atan2(x).to_value())
}

fn log(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 2 {
        return Err(Value::String(Arc::new("Expected 2 arguments".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    let base: f64 = FromPyroValue::from_value(&args[1])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.log(base).to_value())
}

fn log2(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.log2().to_value())
}

fn log10(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::String(Arc::new("Expected 1 argument".to_string())));
    }
    let val: f64 = FromPyroValue::from_value(&args[0])
        .map_err(|e| Value::String(Arc::new(e)))?;
    Ok(val.log10().to_value())
}

fn pi(_args: Vec<Value>) -> Result<Value, Value> {
    Ok(std::f64::consts::PI.to_value())
}

fn e(_args: Vec<Value>) -> Result<Value, Value> {
    Ok(std::f64::consts::E.to_value())
}

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("sqrt".to_string(), Value::NativeFunction {
        name: "sqrt".to_string(),
        func: NativeClosure(Arc::new(sqrt)),
    });
    methods.insert("abs".to_string(), Value::NativeFunction {
        name: "abs".to_string(),
        func: NativeClosure(Arc::new(abs)),
    });
    methods.insert("ceil".to_string(), Value::NativeFunction {
        name: "ceil".to_string(),
        func: NativeClosure(Arc::new(ceil)),
    });
    methods.insert("floor".to_string(), Value::NativeFunction {
        name: "floor".to_string(),
        func: NativeClosure(Arc::new(floor)),
    });
    methods.insert("round".to_string(), Value::NativeFunction {
        name: "round".to_string(),
        func: NativeClosure(Arc::new(round)),
    });
    methods.insert("sin".to_string(), Value::NativeFunction {
        name: "sin".to_string(),
        func: NativeClosure(Arc::new(sin)),
    });
    methods.insert("cos".to_string(), Value::NativeFunction {
        name: "cos".to_string(),
        func: NativeClosure(Arc::new(cos)),
    });
    methods.insert("tan".to_string(), Value::NativeFunction {
        name: "tan".to_string(),
        func: NativeClosure(Arc::new(tan)),
    });
    methods.insert("pow".to_string(), Value::NativeFunction {
        name: "pow".to_string(),
        func: NativeClosure(Arc::new(pow)),
    });
    methods.insert("asin".to_string(), Value::NativeFunction {
        name: "asin".to_string(),
        func: NativeClosure(Arc::new(asin)),
    });
    methods.insert("acos".to_string(), Value::NativeFunction {
        name: "acos".to_string(),
        func: NativeClosure(Arc::new(acos)),
    });
    methods.insert("atan".to_string(), Value::NativeFunction {
        name: "atan".to_string(),
        func: NativeClosure(Arc::new(atan)),
    });
    methods.insert("atan2".to_string(), Value::NativeFunction {
        name: "atan2".to_string(),
        func: NativeClosure(Arc::new(atan2)),
    });
    methods.insert("log".to_string(), Value::NativeFunction {
        name: "log".to_string(),
        func: NativeClosure(Arc::new(log)),
    });
    methods.insert("log2".to_string(), Value::NativeFunction {
        name: "log2".to_string(),
        func: NativeClosure(Arc::new(log2)),
    });
    methods.insert("log10".to_string(), Value::NativeFunction {
        name: "log10".to_string(),
        func: NativeClosure(Arc::new(log10)),
    });
    methods.insert("pi".to_string(), Value::NativeFunction {
        name: "pi".to_string(),
        func: NativeClosure(Arc::new(pi)),
    });
    methods.insert("e".to_string(), Value::NativeFunction {
        name: "e".to_string(),
        func: NativeClosure(Arc::new(e)),
    });

    Value::NativeModule(Arc::new(methods))
}
