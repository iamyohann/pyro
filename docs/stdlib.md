# Pyro Standard Library

Pyro provides a set of standard modules to assist with common tasks such as file system operations, environment manipulation, and more.

## Modules

### `std.fs`
File system operations.
- `create_dir(path: str)`: Create a new directory.
- `remove_dir(path: str)`: Remove a directory.
- `write(path: str, content: str)`: Write string content to a file.
- `read_to_string(path: str) -> str`: Read a file's content as a string.
- `list_dir(path: str) -> [str]`: List entries in a directory.
- `exists(path: str) -> bool`: Check if a path exists.
- `is_file(path: str) -> bool`: Check if a path is a file.
- `is_dir(path: str) -> bool`: Check if a path is a directory.
- `remove_file(path: str)`: Remove a file.

### `std.env`
Environment interaction.
- `cwd() -> str`: Get the current working directory.
- `set_cwd(path: str)`: Set the current working directory.
- `var(name: str) -> str`: Get an environment variable value.
- `vars() -> {str: str}`: Get all environment variables.
- `args() -> [str]`: Get command line arguments.

### `std.path`
Path manipulation utilities.
- `join(parts: [str]) -> str`: Join path components.
- `basename(path: str) -> str`: Get the filename portion of a path.
- `dirname(path: str) -> str`: Get the directory portion of a path.
- `extname(path: str) -> str`: Get the file extension.
- `abs_path(path: str) -> str`: Resolve an absolute path.

### `std.process`
Process control.
- `exec(command: str, args: [str]) -> {stdout: str, stderr: str, code: int}`: Execute a subprocess.
- `exit(code: int)`: Exit the current process.

### `std.json`
JSON handling.
- `stringify(value: any) -> str`: Convert a value to a JSON string.
- `parse(json_str: str) -> any`: Parse a JSON string into a Pyro value.

### `std.random`
Random number generation.
- `random() -> float`: Return a random float between 0.0 and 1.0.
- `randint(min: int, max: int) -> int`: Return a random integer between min and max (inclusive).

### `std.math`
Mathematical functions.
- `abs(x)`, `ceil(x)`, `floor(x)`, `round(x)`
- `sqrt(x)`, `pow(base, exp)`
- `sin(x)`, `cos(x)`, `tan(x)`
- `asin(x)`, `acos(x)`, `atan(x)`, `atan2(y, x)`
- `log(x, base)`, `log2(x)`, `log10(x)`
- `pi() -> float`, `e() -> float`

### `std.time`
Time functions.
- `now() -> float`: Get current timestamp in seconds.
- `millis() -> int`: Get current timestamp in milliseconds.
- `sleep(seconds: float)`: Sleep for the specified duration.
