# Rust Bindings for Pyro

## Objective
Enable Pyro to natively use existing Rust libraries by generating bindings that verify type safety and handle runtime execution within the Pyro interpreter. This avoids reinventing standard library modules (like IO, File System, Math) and allows leveraging the rich Rust ecosystem.

## Investigation Findings

### Current State
*   **Interpreter**: `pyro-core/src/interpreter.rs` handles execution.
*   **Values**: `Value` enum currently only supports `Function` (Pyro user-defined) and `BuiltinMethod` (hardcoded hack for `str.len`, `list.push`, etc.).
*   **Modules**: `Stmt::Import` exists but simply prints a message; there is no real module loading system.
*   **Extensibility**: No current mechanism to inject native functions or modules into the interpreter.

### Architecture Proposal

To support "proxying" Rust libraries, we need to introduce a **Native Interface Layer** (NIL).

#### 1. Extend `Value` Enum
We need a way to represent Rust functions and modules in the runtime.

```rust
pub type NativeFn = dyn Fn(Vec<Value>) -> Result<Value, Value>;

#[derive(Clone)]
pub enum Value {
    // ... existing variants
    NativeFunction {
        name: String,
        func: Rc<NativeFn>,
    },
    NativeModule(Rc<HashMap<String, Value>>),
}
```

#### 2. Native Module Registry
The `Interpreter` should maintain a registry of available native modules.

```rust
pub struct Interpreter {
    globals: HashMap<String, Value>,
    native_modules: HashMap<String, Value>, // e.g., "std::fs" -> Value::NativeModule(...)
}
```

When `import std::fs` is encountered, the interpreter looks up "std::fs" in `native_modules` and binds it to the current scope.

#### 3. Binding Generation (The "Proxy")
Instead of manually writing wrapper functions that parse `Vec<Value>` and handle errors, we should use a declarative approach (likely Rust macros) to generate this glue code.

**Concept: `#[pyro_bind]` Macro**

We want to write something like this in Rust:

```rust
#[pyro_bind]
fn file_read_to_string(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
```

The macro should expand to:

```rust
fn file_read_to_string_wrapper(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 { return Err(make_error("Expected 1 argument")); }
    
    // Type checking & Unpacking
    let path = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => return Err(make_error("Argument 'path' must be a String")),
    };

    // Call implementation
    let result = file_read_to_string(path);

    // Repacking
    match result {
        Ok(s) => Ok(Value::String(Rc::new(s))),
        Err(e) => Err(make_error(&e)),
    }
}
```

#### 4. Type Conversion Trait
To make the macro work, we need a trait for converting between `Value` and Rust types.

```rust
trait FromPyroValue: Sized {
    fn from_value(v: &Value) -> Result<Self, String>;
}

trait ToPyroValue {
    fn to_value(self) -> Value;
}
```

Implementations:
*   `i64` <-> `Value::Int`
*   `String` <-> `Value::String`
*   `bool` <-> `Value::Bool`
*   `Vec<T>` <-> `Value::List` (recursive)

## Detailed Implementation Tasks

### 1. Infrastructure Setup
- [ ] **Update `Value` Enum**
    - Correctly define `NativeFunction` variant in `pyro-core/src/interpreter.rs` (likely needs `Rc<dyn Fn...>`).
    - Add `NativeModule` variant to store module contents (likely `Rc<HashMap<String, Value>>`).
- [ ] **Type Conversion Traits**
    - Create `pyro-core/src/convert.rs`.
    - Implement `trait FromPyroValue` to convert `Value` -> Rust types (`i64`, `f64`, `String`, etc.).
    - Implement `trait ToPyroValue` to convert Rust types -> `Value`.
- [ ] **Native Module Registry**
    - Add `native_modules: HashMap<String, Value>` to `Interpreter` struct.
    - Create a helper method `register_native_module` to easily add modules.

### 2. Interpreter Integration
- [ ] **Update `Stmt::Import`**
    - Modify `execute_stmt` for `Stmt::Import`.
    - Logic: Check `self.native_modules`. If found, insert into `self.globals` using the module name (or alias).
    - If not found, fall back to existing (stub) behavior or filesystem loading (if implemented).

### 3. Proof of Concept: Math Module
- [ ] **Create `pyro-core/src/stdlib/math.rs`**
    - Implement `fn sqrt(args: Vec<Value>) -> Result<Value, Value>`.
    - Use `FromPyroValue` to unpack arguments.
- [ ] **Register Module**
    - In `Interpreter::new()`, register "std.math".
- [ ] **Verification**
    - Create `tests/stdlib_math.pyro`.
    - Test `import std.math`, `std.math.sqrt(144)`.

### 4. Proof of Concept: IO Module
- [ ] **Create `pyro-core/src/stdlib/io.rs`**
    - Implement `fn read_file(path: String)`.
    - Implement `fn write_file(path: String, content: String)`.
- [ ] **Register Module**
    - Register "std.io".
- [ ] **Verification**
    - Create `tests/stdlib_io.pyro`.
    - Test reading and writing a temporary file.

## Example: `std::math`

**Rust Implementation:**
```rust
pub fn sqrt(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 { return Err(make_val_error("args")); }
    let val = match &args[0] {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return Err(make_val_error("number")),
    };
    Ok(Value::Float(val.sqrt()))
}
```

**Pyro Usage:**
```python
import std.math
print(math.sqrt(16)) # Output: 4.0
```
