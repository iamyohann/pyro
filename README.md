# Pyro Programming Language

> ⚠️⚠️⚠️ **EXPERIMENTAL WARNING**
> This is an experimental programming language. It is not ready for production use. **DO NOT USE IN PRODUCTION**

Pyro is a new programming language designed to be as simple as Python but with the performance and safety of Rust and robustness of some aspects of GoLang (package management, concurrency model etc...).


## Installation

### Homebrew

```bash
brew tap iamyohann/pyro https://github.com/iamyohann/pyro
brew install --HEAD pyro
```

## Example usage

```bash
cargo run -p pyro-cli -- run examples/hello.pyro
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

## Features
- **Syntax**: Python-like (indentation-based).
- **Type System**: Strong, static typing with inference (Rust-inspired).
- **Functional**: Automatic currying, partial application, and immutable data structures.
- **Memory**: Automatic memory management (ARC/GC) for ease of use.
- **Tooling**: Built-in package manager (Git-based semantics).
- **Native Modules**: Extend Pyro with Rust.
