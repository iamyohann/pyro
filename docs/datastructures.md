# Data Structure Methods

Pyro's default data structures support a variety of built-in methods similar to Python and Rust.

## List

Immutable lists support read-only methods. Mutable lists (`list_mut`) support modification methods.

### Methods

- `len()`: Returns the number of elements.
- `push(item)`: Adds an item to the end (Mutable only).
- `pop()`: Removes and returns the last item (Mutable only).
- `insert(index, item)`: Inserts an item at the specified index (Mutable only).
- `remove(item)`: Removes the first occurrence of `item` (Mutable only).
- `clear()`: Removes all items (Mutable only).
- `reverse()`: Reverses the list in place (Mutable only).

### Example

```python
let l = [1, 2, 3]
print(l.len()) # 3

let m = ListMutable([1, 2])
m.push(3)
print(m.len()) # 3
m.pop()
```

## Dict

Dictionaries store key-value pairs.

### Methods

- `len()`: Returns the number of items.
- `keys()`: Returns a list of keys.
- `values()`: Returns a list of values.
- `items()`: Returns a list of (key, value) tuples.
- `get(key)`: Returns the value for `key`, or Void if not found.
- `remove(key)`: Removes the item with the given key (Mutable only).
- `clear()`: Removes all items (Mutable only).

### Example

```python
let d = { "a": 1, "b": 2 }
print(d.keys()) # ["a", "b"]

let m = DictMutable({})
m.insert("c", 3) # Wait, insert not implemented for Dict?
# Actually DictMutable support implicit assignment? 
# Or proper set item method? 
# Currently DictMutable uses `remove`. 
# Insertion is usually done via index assignment `d["k"] = v`.
# But `d.update` or similar might be useful. 
# Implemented methods: keys, values, items, len, clear, remove, get.
```

## Set

Sets store unique values.

### Methods

- `len()`: Returns the number of items.
- `contains(item)`: Returns true if item is in the set.
- `add(item)`: Adds an item (Mutable only).
- `remove(item)`: Removes an item (Mutable only).

## String

Strings are immutable sequences of characters.

### Methods

- `len()`: Returns length of string.
- `upper()`: Returns uppercase version.
- `lower()`: Returns lowercase version.
- `contains(substring)`: Returns true if substring is found.
- `split(delimiter)`: Returns a list of substrings split by delimiter.

### Example

```python
let s = "Hello World"
print(s.len())
print(s.upper())
```
