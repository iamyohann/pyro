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
let c = chan<int>(1) // Buffered channel of integers with capacity 1
```

### Sending and Receiving

Use the `.push(value)` and `.collect()` methods to interact with channels.

-   **Send**: `channel.push(value)`
-   **Receive**: `val = channel.collect()`

```python
import std.time

def worker(c):
    std.time.sleep(1.0)
    print("Worker sending")
    c.push("Done")

let c = chan<string>(1)
go worker(c)

print("Waiting...")
let msg = c.collect()
print("Received: " + msg)
```

### Directional Channels

You can restrict a channel to be send-only or receive-only using `.sender()` and `.receiver()` methods.

```python
let c = chan<int>(1)
let tx = c.sender()   // Send-only
let rx = c.receiver() // Receive-only

tx.push(1)
// tx.collect() // Error: Channel is send-only

let val = rx.collect()
// rx.push(2) // Error: Channel is receive-only
```

Channels are safe to share across threads. Sending to a full channel blocks the sender (suspends the task), and receiving from an empty channel blocks the receiver.


## Configuration

The size of the thread pool can be configured via the `PYRO_WORKER_THREADS` environment variable.

```bash
# Run with 4 worker threads
PYRO_WORKER_THREADS=4 cargo run -p pyro-cli -- run script.pyro
```


## Concurrency Patterns

### Orchestrator and Worker

Common pattern where a main thread (orchestrator) sends jobs to a queue, and multiple worker threads process them.

```python
import std.time

def worker(id: int, jobs: Receiver<int>, results: Sender<int>):
    while true:
        let job = jobs.collect()
        if job == -1: # Termination signal
            break
        
        # Process job
        std.time.sleep(0.1)
        results.push(job * 2)

let jobs = chan<int>(10)
let results = chan<int>(10)

# Start workers
go worker(1, jobs.receiver(), results.sender())
go worker(2, jobs.receiver(), results.sender())

# Send jobs
for i in 0..5:
    jobs.push(i)

# Terminate workers
jobs.push(-1)
jobs.push(-1)

# Collect results
for i in 0..5:
    print(results.collect())
```

### Fan-In

Multiple producers sending data to a single channel.

```python
def producer(id: int, out: Sender<string>):
    out.push("Producer " + str(id))

let c = chan<string>(10)
let tx = c.sender()

go producer(1, tx)
go producer(2, tx)
go producer(3, tx)

for i in 0..3:
    print(c.collect())
```
