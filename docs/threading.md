# Threading & Concurrency

Pyro supports concurrency through lightweight threads, similar to Go's goroutines. These tasks are scheduled onto a pool of worker threads managed by the underlying Tokio runtime.

## The `go` Keyword

To spawn a new task, use the `go` keyword followed by a function call:

```python
import std.time

def worker(id: int):
    print("Worker " + str(id) + " started")
    std.time.sleep(1.0)
    print("Worker " + str(id) + " finished")

print("Main starting")
go worker(1)
go worker(2)
std.time.sleep(2.0)
print("Main done")
```

The `go` statement takes a function call expression. It evaluates the arguments in the current thread, and then executes the function body concurrently.

### Shared State

Pyro uses `Arc` (Atomic Reference Counting) and `RwLock` (Read-Write Lock) internally to ensure memory safety when multiple threads access the same data.

- **Immutable Data** (e.g., `List`, `Dict` default): Safe to read from multiple threads.
- **Mutable Data** (e.g., `list_mut`, `dict_mut`, `Instance` fields): Protected by implicit locks.

## Channels

Pyro provides channels for synchronization and communication between threads, similar to Go.

### Creating a Channel

Use the `chan(capacity)` built-in function to create a channel.

```python
let c = chan(1) // Buffered channel with capacity 1
```

### Sending and Receiving

Use the `<-` operator to send and receive values.

-   **Send**: `channel <- value` (Statement)
-   **Receive**: `val = <- channel` (Expression)

```python
import std.time

def worker(c):
    std.time.sleep(1.0)
    print("Worker sending")
    c <- "Done"

let c = chan(1)
go worker(c)

print("Waiting...")
let msg = <- c
print("Received: " + msg)
```

Channels are safe to share across threads. Sending to a full channel blocks the sender (suspends the task), and receiving from an empty channel blocks the receiver.


## Configuration

The size of the thread pool can be configured via the `PYRO_WORKER_THREADS` environment variable.

```bash
# Run with 4 worker threads
PYRO_WORKER_THREADS=4 cargo run -p pyro-cli -- run script.pyro
```

If not specified, it defaults to the number of logical CPUs on your machine.
