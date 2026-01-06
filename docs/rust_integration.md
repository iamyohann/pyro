# Rust Integration

Pyro allows you to seamlessly integrate Rust packages and native code into your projects. This feature enables you to leverage the performance and vast ecosystem of Rust directly from your Pyro scripts.

## Adding Rust Dependencies

To use a Rust crate, add a `[rust]` section to your `pyro.mod` file and list the dependencies just like you would in a `Cargo.toml`.

```toml
[package]
name = "my_project"
version = "0.1.0"

[rust]
rand = "0.8"
base64 = "0.21"
```

## Auto-Generated Bindings

The easiest way to call Rust code is to use Pyro's auto-binding generation. You declare an `extern` function with a string literal pointing to the Rust function's path. Pyro will automatically generate the glue code to marshal arguments and return values.

### Example

```python
# main.pyro
extern "rand::random" def rand_float() -> float
extern "base64::encode" def b64_encode(input: string) -> string

let x = rand_float()
print(x)

let encoded = b64_encode("hello world")
print(encoded)
```

The Pyro CLI (`pyro run`) handles the rest:
1.  Detects `[rust]` dependencies in `pyro.mod`.
2.  Scans your code for `extern "path" def ...`.
3.  Generates a Rust wrapper project in `~/.pyro/rustpkg/<project_hash>`.
4.  Compiles and runs your code with the native extensions linked.

### Supported Types

Currently, the auto-generator supports mapping the following primitive types:

| Pyro Type | Rust Type |
| :--- | :--- |
| `int` | `i64` |
| `float` | `f64` |
| `bool` | `bool` |
| `string` | `String` |
| `void` | `()` |

## Manual Native Functions (Advanced)

For more complex logic that requires manual argument parsing or state management, you can still define a `native.rs` file in your project root.

### 1. Create `native.rs`

Define your public Rust functions with the signature `fn(Vec<Value>) -> Result<Value, Value>`.

```rust
// native.rs
use pyro_core::interpreter::Value;

pub fn my_complex_func(args: Vec<Value>) -> Result<Value, Value> {
    // Manual argument handling
    Ok(Value::Bool(true))
}
```

### 2. Declare in Pyro

```python
extern def my_complex_func() -> bool
```
