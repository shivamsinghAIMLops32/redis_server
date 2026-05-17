# Mini-Redis Server in Rust

A lightweight, multi-threaded, in-memory key-value store built from scratch in Rust. This project mimics the behavior of a simple Redis server, supporting concurrent client connections, basic caching commands (`SET`, `GET`, `PING`), and Time-To-Live (TTL) functionality using a lazy-expiration strategy.

---

## 🏗️ Architecture & Step-by-Step Implementation

### Step 1: TCP Server Setup & Port Binding

To allow clients to communicate with our DB, we first open a port using `TcpListener`. The server listens continuously for incoming connections on port `6379` (the default Redis port).

```rust
let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
println!("Mini-Redis is listening on port 6379!");
```

### Step 2: Accepting Concurrent Connections

When a client connects, we receive a `TcpStream`. To handle multiple clients at once without blocking the main server, we spawn a new thread for each connection using `thread::spawn`.

```rust
for stream in listener.incoming() {
    match stream {
        Ok(mut stream) => {
            // Clone the Arc pointer for safe shared memory access
            let db_clone = Arc::clone(&db);

            thread::spawn(move || {
                // Thread-specific connection loop
            });
        }
        Err(_) => println!("Connection failed"),
    }
}
```

### Step 3: Reading Data into a Buffer

Inside the thread loop, we create a buffer `[0; 512]` to accept data from the TCP stream. The `stream.read` method pulls the raw bytes sent by the user into the buffer. Once read, we convert these bytes into a usable UTF-8 string.

```rust
let mut buffer = [0; 512];
match stream.read(&mut buffer) {
    Ok(0) => break, // Client disconnected
    Ok(size) => {
        let raw_text = String::from_utf8_lossy(&buffer[..size]);
        // ... parse command
    }
    // ...
}
```

### Step 4: Command Parsing

To make sense of the text inputs, the raw string is handed over to a custom parser. Using an `enum` and `.split_whitespace()`, we isolate the command and its arguments.

```rust
pub enum Command {
    Ping,
    Set(String, String),
    SetEx(String, u64, String),
    Get(String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
    let mut parts = input.trim().split_whitespace();
    let cmd = parts.next().unwrap_or("").to_uppercase();
    // Match cases for PING, SET, SETEX, GET...
}
```

### Step 5: Shared State In-Memory Database (Arc & Mutex)

The core database is a standard Rust `HashMap`. Because multiple threads need to read and write to this map simultaneously, it is wrapped in an `Arc` (Atomic Reference Counted pointer) and a `Mutex` (Mutual Exclusion lock) for thread safety.

```rust
struct DbValue {
    data: String,
    expires_at: Option<SystemTime>,
}

// In main.rs:
let db = Arc::new(Mutex::new(HashMap::<String, DbValue>::new()));
```

Whenever a command requires accessing the DB, the thread locks the Mutex, modifies the map, and releases the lock as soon as the scope ends.

### Step 6: Implementing TTL with "Lazy Expiration"

Instead of spawning a complex background thread that constantly drains CPU by checking for expired keys, we use a **Lazy Expiration** approach.

When a `SETEX` command is received, we calculate the exact `SystemTime` it should expire and save it in the DB.
When a `GET` command occurs, we first check the key's timestamp against the current clock. If it's expired, we seamlessly delete it right then and return a blank payload (`$-1\r\n`).

```rust
// Inside GET handler:
let mut should_delete = false;

if let Some(db_value) = map.get(&key) {
    if let Some(expiration) = db_value.expires_at {
        if SystemTime::now() > expiration {
            should_delete = true;
        }
    }
}

if should_delete {
    map.remove(&key);
}
```

### Step 7: Replying to the Client

Every successful computation pushes a response string back to the user via `stream.write_all(response.as_bytes())`. For example, `SET` returns `+OK\r\n`, whereas an unknown command replies with `-ERR unknown command...`.
