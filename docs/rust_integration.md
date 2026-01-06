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

## Using Native Functions

You can define native functions in a `native.rs` file in your project root. These functions can be essentially "linked" to Pyro using the `extern` syntax.

### 1. Create `native.rs`
Create a `native.rs` file and define your public Rust functions. They must follow the signature `fn(Vec<Value>) -> Result<Value, Value>`.

```rust
// native.rs
use pyro_core::interpreter::Value;

pub fn rand_float(_args: Vec<Value>) -> Result<Value, Value> {
    let x: f64 = rand::random();
    Ok(Value::Float(x))
}
```

### 2. Declare in Pyro
In your `main.pyro` (or any Pyro file), use the `extern` keyword to declare these functions. Pyro's build system will automatically generate the bindings for you.

```python
extern def rand_float() -> float

let x = rand_float()
print(x)
```

## How it Works

When you run `pyro run`, the CLI detects the `[rust]` dependencies and the `native.rs` file. It then:
1.  Creates a temporary Rust project in `~/.pyro/rustpkg`.
2.  Generates a custom runner that includes your dependencies and registers your native functions.
3.  Compiles and runs the project using `cargo`.

This process is transparent, allowing you to mix high-level Pyro code with low-level Rust performance effortlessly.
