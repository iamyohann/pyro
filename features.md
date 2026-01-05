# Pyro Features Tracker

Tracking feature parity with Python and new additions.

## 1. Control Flow
- [x] **For Loops**: Implement `for item in iterable:` syntax.
- [x] **Range Function**: Implement `range(start, stop, step)` for efficient iteration.
- [ ] **Break/Continue**: Verify support in loops.

## 2. Object Oriented Programming (OOP)
- [x] **Classes**: Full support for `class` keyword.
    - [x] `__init__` constructor.
    - [x] `self` reference.
    - [x] Method definitions within classes.
    - [ ] Inheritance (`class Child(Parent):`).
- [x] **Objects**: Instantiation of classes `obj = MyClass()`.

## 3. Error Handling
- [ ] **Try...Except**: Implement `try`, `except`, `finally` blocks for error handling.
- [ ] **Raise**: Ability to raise exceptions.

## 4. Built-in Functions
- [ ] **Global Functions**: Implement global versions of common methods to match Python style.
    - [ ] `len(x)` (wraps `x.len()`).
    - [ ] `type(x)`
    - [ ] `str(x)`, `int(x)`, `float(x)` (Casting functions).
    - [ ] `input(prompt)` (User input).

## 5. String Manipulation
- [ ] **String Formatting**: Implement f-strings `f"Value: {x}"` or `.format()`.
- [ ] **Slicing**: Support `string[start:end]` syntax.
- [ ] **Multiline Strings**: Triple quotes `"""..."""`.

## 6. Standard Library Modules
- [ ] **Math Module**: `import math` (ceil, floor, sqrt, pi, etc.).
- [ ] **Datetime Module**: `import datetime`.
- [ ] **JSON Module**: `import json` (parse/stringify).
- [ ] **RegEx Module**: `import re`.
- [ ] **File I/O**: `open()`, `read()`, `write()`, `close()`.

## 7. Advanced Syntax (Lower Priority)
- [ ] **Lambdas**: `lambda a, b: a + b`.
- [ ] **List Comprehensions**: `[x for x in list]`.
- [ ] **Delete**: `del` keyword.
- [ ] **Pass**: `pass` keyword.
