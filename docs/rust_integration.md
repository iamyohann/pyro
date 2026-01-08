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
hex = "0.4"
```

## Consuming Rust Crates

When you run `pyro run` (or `pyro build`), Pyro automatically:
1.  Downloads and compiles the Rust dependencies.
2.  Generates Pyro extern definitions for them in a `.externs` directory.
3.  Links them into your project.

You can then import them using the special `extern.` prefix syntax:

```python
import extern.hex
import extern.rand

# Functions are namespaced under the import path
let encoded = extern.hex.encode("hello world")
print(encoded)
```

## Naming Conventions

The generated bindings follow these conventions:

-   **Import Path**: `extern.<crate_name>` (e.g., `extern.hex`, `import extern.rand`).
-   **Usage**: `extern.<crate_name>.<function_name>` (e.g., `extern.hex.encode`).

### Generics

For generic Rust functions (like `rand::random<T>()`), Pyro generates specialized variants for common primitive types by appending the type name:

-   `random<f64>` -> `extern.rand.random_float()`
-   `random<i64>` -> `extern.rand.random_int()`
-   `random<bool>` -> `extern.rand.random_bool()`

```python
let f = extern.rand.random_float()
let i = extern.rand.random_int()
```

## Generation CLI

The automated generation happens implicitly during run/build. However, if you wish to manually trigger it to inspect the generated files (located in `.externs/`), you can run:

```bash
pyro externs
```

## Supported Types

Currently, the auto-generator supports mapping the following primitive types. Rust functions utilizing other types will be skipped or commented out in the generated file.

| Pyro Type | Rust Type |
| :--- | :--- |
| `int` | `i64`, `i32`, `u64`, `u32` |
| `float` | `f64`, `f32` |
| `bool` | `bool` |
| `string` | `String`, `str`, `Vec<u8>`, `&[u8]` |
| `void` | `()` |

## Manual Native Functions (Advanced)

For more complex logic that requires manual argument parsing, state management, or unsupported types, you can still define a `native.rs` file in your project root.

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
