# Generics in Pyro

Pyro supports Generics, allowing you to write flexible, reusable code that works with any data type.

## Generic Functions

You can define functions that accept generic type parameters.

```python
def identity<T>(x: T) -> T:
    return x

let n = identity(42)
let s = identity("hello")
```

Multiple type parameters are separated by commas:

```python
def pair<T, U>(first: T, second: U):
    # ...
```

## Generic Records

Records can also be generic.

```python
record Box<T>(value: T)

let b = Box(10)
```
let int_box: Box<int> = { "value": 10 }
let str_box: Box<string> = { "value": "hello" }
```

## Generic Interfaces

Interfaces can specify generic type parameters.

```python
interface Container<T> {
    def get() -> T
    def set(val: T)
}
```

## Generic Type Aliases

Type aliases can be generic too.

```python
type Callback<T> = (T) -> void
```
