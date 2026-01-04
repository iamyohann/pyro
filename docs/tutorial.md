# Pyro Tutorial

Welcome to Pyro! Pyro is a simple, compiled-like language with Python-like syntax and Go-like package management.

## Installation

### macOS (Homebrew)

```bash
brew tap iamyohann/pyro
brew install pyro
```

## Getting Started

### 1. Initialize a Project

Create a new directory for your project and initialize it:

```bash
mkdir my_project
cd my_project
pyro mod init my_project
```

This creates a `pyro.mod` file and a `src/main.pyro` file.

### 2. Run Your Code

Run the project:

```bash
pyro run src/main.pyro
```

You should see: `Hello, Pyro!`

## Language Basics

### Variables

```python
let x: int = 42
let y: float = 3.14
let name: string = "Pyro"
mut z: int = 10  # Mutable variable
z = 11
```

### Functions

```python
def add(a: int, b: int) -> int:
    return a + b

print(add(5, 10))
```

### Control Flow

```python
if x > 10:
    print("Big check")
else:
    print("Small check")

while x > 0:
    x = x - 1
```

## Package Management

Pyro uses a distributed package management system similar to Go.

### Installing Packages

To use a library hosted on GitHub:

```bash
pyro get github.com/username/repo
```

### Importing Code

You can import local files or installed packages:

```python
# Import local file
import "utils.pyro"

# Import installed package
import "github.com/username/repo/src/lib.pyro"

lib_function()
```

For more details on packages, see [Package Management](packages.md).
