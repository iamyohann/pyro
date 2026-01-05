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

### 3. Compile to Binary

To compile your Pyro project into a highly efficient native binary:

```bash
pyro build src/main.pyro
```

This generates a standalone executable (e.g., `./main` or named after the file) that you can run directly without the pyro CLI:

```bash
./main
```

You can also specify the output filename using the `--output` (or `-o`) flag:

```bash
pyro build src/main.pyro --output my_app
pyro build src/main.pyro --output my_app
./my_app
```

### 4. Transpile to Rust

You can also transpile your Pyro code to Rust for debugging or integration purposes:

```bash
pyro build src/main.pyro --target rust --output app.rs
```

This will generate a Rust source file that contains the transpiled code.

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

# Break and Continue
mut i = 0
while i < 10:
    i = i + 1
    if i == 5:
        continue # Skip 5
    if i > 8:
        break    # Stop loop
    print(i)
```


### Data Structures

Pyro supports lists, tuples, sets, and dictionaries. By default, these structures are immutable.

#### Immutable Structures

```python
# List (Immutable)
let my_list: list = [1, 2, 3]
# my_list[0] = 5  # Error: Immutable

# Tuple (Immutable)
let my_tuple: tuple = (1, 2, 3)

# Set (Immutable)
let my_set: set = {1, 2, 3}

# Dictionary (Immutable)
let my_dict: dict = {"key": "value"}
```

#### Mutable Structures

To use mutable versions, use the `Mutable` constructors:

```python
# Mutable List
let mut_list: list_mut = ListMutable([1, 2, 3])

# Mutable Tuple
let mut_tuple: tuple_mut = TupleMutable((1, 2))

# Mutable Set
let mut_set: set_mut = SetMutable({1, 2})

# Mutable Dict
let mut_dict: dict_mut = DictMutable({"key": "value"})
```


#### Complex Examples

You can nest data structures and mix types freely:

```python
# Nested immutable list
let nested: list = [1, [2, 3], 4]

# List with mixed types
let mixed: list = [1, "two", 3.0, true]

# List of dictionaries
let users: list = [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
]

# Nested Mutable Structure (Note: The outer container is mutable, inner ones are immutable unless specified)
let complex_mut: list_mut = ListMutable([
    {"x": 1, "y": 2},
    {"x": 3, "y": 4}
])
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

## Type System & OOP

Pyro supports modern type system features including Structs, Interfaces, and Type Aliases.

### Type Aliases

Use `type` to create cleaner names for complex types.

```python
type UserID = int
let id: UserID = 12345
```

### Records

Define the shape of your data using `record`. Records are immutable.

```python
record Point(x: int, y: int)

let p = Point(10, 20)
print(p.x)
```

Records can also have methods:

```python
record Rect(w: int, h: int):
    def area(self) -> int:
        return self.w * self.h

let r = Rect(10, 5)
print(r.area())
```

### Interfaces

Define behavior contracts using `interface`. Pyro interfaces are satisfied implicitly.

```python
interface Printer {
    def print_details() -> string
}

# If a struct has a method 'print_details() -> string', 
# it can be used wherever a 'Printer' is expected.
```

For a complete reference, check out [Type System Reference](types.md).
