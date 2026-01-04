# Pyro Type System Reference

Pyro is a statically typed language with type inference. This document details the available types and how to use them.

## Basic Types

| Type | Description | Example |
|------|-------------|---------|
| `int` | Signed 64-bit integer | `42`, `-1` |
| `float` | 64-bit floating point | `3.14`, `-0.01` |
| `bool` | Boolean value | `true`, `false` |
| `string` | UTF-8 string | `"Hello"` |
| `void` | Absence of value | `return` |

## Collection Types

By default, all collections in Pyro are **immutable**.

### List
Ordered collection of elements.
- Type: `list`
- Mutable: `list_mut`

```python
let nums: list = [1, 2, 3]
let mut_nums: list_mut = ListMutable([1, 2, 3])
```

### Tuple
Fixed-size ordered collection.
- Type: `tuple`
- Mutable: `tuple_mut`

```python
let point: tuple = (10, 20)
```

### Set
Unordered collection of unique elements.
- Type: `set`
- Mutable: `set_mut`

```python
let unique: set = {1, 2, 3}
```

### Map / Dictionary
Key-value pairs.
- Type: `dict`
- Mutable: `dict_mut`

```python
let scores: dict = {"Alice": 100, "Bob": 90}
```

## Advanced Types

### Type Aliases
You can create an alias for any existing type to improve code readability.

```python
type MyInt = int
type ID = string

let user_id: ID = "u-123"
```

### Structs
Structs allow you to define custom data structures with named fields.

```python
struct User {
    id: int
    name: string
    active: bool
}

# Instantiation (Not yet implemented, conceptual syntax)
# let u = User { id: 1, name: "Alice", active: true }
```

### Interfaces
Interfaces define a contract of behavior. Pyro uses **implicit satisfaction** (duck typing), similar to Go. Use interfaces to define the methods a type must implement.

```python
interface Reader {
    def read(size: int) -> string
}

interface Writer {
    def write(data: string) -> int
}

# Any type (e.g. a struct) that has a 'read' method with the correct signature 
# automatically satisfies the 'Reader' interface.
```
