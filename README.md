# Pyro Programming Language

> ⚠️⚠️⚠️ **EXPERIMENTAL WARNING**
> This is an experimental programming language. It is not ready for production use. **DO NOT USE IN PRODUCTION**

Pyro is a new programming language designed to be as simple as Python but with the performance and safety of Rust and robustness of some aspects of GoLang (package management, concurrency model etc...).


## Installation


```bash
### Clone the repository
git clone https://github.com/iamyohann/pyro.git

cd pyro

### Build the CLI
cargo build --release

## Example usage

```bash
cargo run -p pyro-cli -- run examples/hello.pyro
```

Shell

```bash
cargo run -p pyro-cli -- shell
```


## Tutorial

See [docs/tutorial.md](docs/tutorial.md).

## Documentation
- [Tutorial](docs/tutorial.md)
- [Type System Reference](docs/types.md)
- [Generics](docs/generics.md)
- [Functional Programming](docs/functional.md)
- [Package Management](docs/packages.md)
- [Data Structures](docs/datastructures.md)
- [Error Handling](docs/error_handling.md)
- [Native Bindings](docs/native_bindings.md)
- [Standard Library](docs/stdlib.md)
- [Threading & Concurrency](docs/threading.md)

## Features
- **Syntax**: Python-like (indentation-based).
- **Type System**: Strong, static typing with inference (Rust-inspired).
- **Functional**: Automatic currying, partial application, and immutable data structures.
- **Concurrency**: Go-style concurrency with `go` routines.
- **Memory**: Automatic memory management (ARC) for ease of use.
- **Tooling**: Built-in package manager (Git-based semantics).
- **Native Bindings**: Easily bind C/Rust libraries.
- **Rust Integration**: Import and use Rust crates directly. [Learn more](docs/rust_integration.md).
