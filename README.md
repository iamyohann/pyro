# Pyro Programming Language

> ⚠️⚠️⚠️ **EXPERIMENTAL**
> This is an experimental programming language. It is not ready for production use. This project tests the ability for LLMs to generate compilers for new programming languages.

Pyro is a new programming language designed to be as simple as Python but with the performance and safety of Rust.

## Features (Planned)
- **Syntax**: Python-like (indentation-based).
- **Type System**: Strong, static typing with inference (Rust-inspired).
- **Memory**: Automatic memory management (ARC/GC) for ease of use.
- **Tooling**: Built-in package manager (Git-based semantics).

## Example usage

```bash
cargo run -p pyro-cli -- run examples/hello.pyro
```

## Tutorial

See [docs/tutorial.md](docs/tutorial.md).