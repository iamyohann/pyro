# Error Handling in Pyro

Pyro provides robust error handling mechanisms including `try-except` blocks, exception raising, custom error types through inheritance, and error chaining.

## Try, Except, Finally

You can handle exceptions using `try`, `except`, and `finally` blocks.

```python
try:
    # Code that might raise an error
    x = 1 / 0
except e:
    # Handle the error
    print("Caught error: " + e.message)
finally:
    # Cleanup code (always runs)
    print("Cleanup")
```

The `except` block catches any error raised in the `try` block. The variable (e.g., `e`) is bound to the error instance.

## Raising Exceptions

You can raise exceptions using the `raise` keyword. Exceptions in Pyro are objects, typically instances of the built-in `Error` class or its subclasses.

```python
raise Error("Something went wrong")
```

## The `Error` Class

Pyro has a built-in `Error` class which serves as the base class for all exceptions. It has a `message` field.

```python
class Error:
    def __init__(self, message: String):
        self.message = message
```

## Custom Errors

You can define your own error types by inheriting from the `Error` class.

```python
class meaningfulError(Error):
    def __init__(self, msg, code):
        self.message = msg
        self.code = code

try:
    raise meaningfulError("Resource not found", 404)
except e:
    print(e.code) # Prints 404
```

## Error Chaining

When handling an error, you might want to raise a new error while preserving the original cause. You can do this using `raise ... from ...`.

```python
try:
    read_file()
except io_error:
    raise Error("Failed to load config") from io_error
```

The original error is stored in the `cause` field of the new error.

```python
try:
    # ...
except e:
    if e.cause:
        print("Original cause: " + e.cause.message)
```
