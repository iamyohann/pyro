# Functional Programming in Pyro

Pyro supports key functional programming concepts, making it easy to write expressive and concise code.

## Automatic Currying

All functions (and constructors) in Pyro are automatically curried. This means if you call a function with fewer arguments than it expects, it returns a new function that accepts the remaining arguments.

```python
def add(x: int, y: int) -> int:
    return x + y

let add5 = add(5)    # Returns a function waiting for 'y'
print(add5(10))      # Prints 15
```

This applies to any number of arguments:

```python
def sum3(x: int, y: int, z: int) -> int:
    return x + y + z

let step1 = sum3(1)
let step2 = step1(2)
let result = step2(3) # 6
```

## Immutable Data

By default, Pyro data structures are immutable. This aligns with functional programming principles, preventing accidental side effects.

- **Records**: Immutable by default.
- **Lists/Maps**: Immutable by default (use `ListMutable` etc. for mutability).

```python
record Point(x: int, y: int)
let p = Point(1, 2)
# p.x = 3  <-- Error: Cannot modify immutable record
```

## Record Constructors as Functions

Since constructing a record is just a function call, you can curry constructors too:

```python
record User(id: int, name: string)

let makeUser1 = User(1)
let bob = makeUser1("Bob")
```

## Method Currying

Methods on records are also curried. The first argument is implicitly `self`, but subsequent arguments work just like standard functions.

```python
record Calculator(factor: int):
    def add(self, a: int, b: int) -> int:
        return self.factor + a + b

let calc = Calculator(10)
let add5 = calc.add(5)  # Returns function waiting for 'b'
print(add5(5))          # 20
```
