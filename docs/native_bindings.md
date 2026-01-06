# Native Bindings in Pyro

Pyro allows extending the language with native Rust functions and modules. This is achieved through the Native Interface Layer.

> [!TIP]
> For simple integrations with existing Rust crates, check out [Rust Integration](rust_integration.md) for an easier, automated workflow.


## Structure

Native modules are registered in the `Interpreter` via the `native_modules` registry. A native module is simply a `Value::NativeModule` which contains a map of export names to `Value`s (typically `Value::NativeFunction`).

## Implementing a Native Module

1. Define your Rust functions. They must match the signature:
   ```rust
   fn my_func(args: Vec<Value>) -> Result<Value, Value>
   ```
2. Use `FromPyroValue` trait to convert arguments from Pyro `Value` to Rust types.
3. Use `ToPyroValue` trait (or manual conversion) to return a `Value`.
4. Wrap your function in `NativeClosure` and put it in a `Value::NativeFunction`.
5. Bundle functions into a `HashMap` and return `Value::NativeModule`.

### Example: std.math

```rust
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

pub fn module() -> Value {
    let mut methods = HashMap::new();
    
    methods.insert("sqrt".to_string(), Value::NativeFunction {
        name: "sqrt".to_string(),
        func: NativeClosure(Rc::new(sqrt)),
    });

    Value::NativeModule(Rc::new(methods))
}
```

## Registering the Module

In `pyro-core/src/stdlib/mod.rs` (or where you initialize the interpreter):

```rust
pub fn register_std_libs(interpreter: &mut Interpreter) {
    interpreter.register_native_module("std.math", math::module());
    // ...
}
```

## Using in Pyro

```python
import std.math

let x = math.sqrt(16)
print(x) // 4.0
```

## Error Handling

Native functions should return `Err(Value)` if an error occurs. Typically this is `Value::String`. The interpreter automatically wraps `Value::String` errors into Pyro `Error` objects, allowing them to be caught with `try/except` blocks.

```python
try:
    math.sqrt("invalid")
except e:
    print(e.message)
```
