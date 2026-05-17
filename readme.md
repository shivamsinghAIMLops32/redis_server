# Mini-Redis Server in Rust

A lightweight, multi-threaded, in-memory key-value store built from scratch in Rust. This project mimics the behavior of a simple Redis server, supporting concurrent client connections, caching commands (`SET`, `GET`, `PING`), Time-To-Live (TTL) functionality, and **Publish/Subscribe (Pub/Sub)** messaging.

---

## 🏗️ Architecture & Step-by-Step Implementation

### Step 1: Modularity & Database State
We extracted the core logic into its own module `src/db.rs` to keep things clean. The `Database` struct holds two maps securely wrapped in `Arc` and `Mutex` to be totally thread-safe:
1. `store`: Holds our key-value caching data.
2. `pubsub`: A registry matching channel names to vectors of active `TcpStream` client connections.

```rust
// src/db.rs
#[derive(Clone)]
pub struct Database {
    store: Arc<Mutex<HashMap<String, DbValue>>>,
    pubsub: Arc<Mutex<HashMap<String, Vec<TcpStream>>>>,
}
```

### Step 2: TCP Server Setup & Port Binding

To allow clients to communicate with our DB, we open a port using `TcpListener`. The server listens continuously for incoming connections on port `6379`.

```rust
let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
println!("Mini-Redis is listening on port 6379!");
```

### Step 3: Accepting Concurrent Connections

When a client connects, we receive a `TcpStream`. To handle multiple clients at once without blocking the main server, we spawn a new thread for each connection. We also clone the inner data states (cheaply) to share them correctly across threads.

```rust
for stream in listener.incoming() {
    match stream {
        Ok(mut stream) => {
            let db_clone = db.clone(); 
            thread::spawn(move || {
                // Thread-specific loop...
            });
        }
        Err(e) => println!("Failed to connect: {}", e),
    }
}
```

### Step 4: Reading Data & Command Parsing

Inside the loop, we buffer data coming from the user and parse the string logic via our custom `parser.rs`.

```rust
// src/parser.rs
pub enum Command {
    Ping,
    Set(String, String),
    SetEx(String, u64, String),
    Get(String),
    Subscribe(String),
    Publish(String, String),
    Unknown(String),
}
```

We split inputs using `.split_whitespace()` iteratively, transforming commands into `Enum` variants.

### Step 5: Implementing TTL with "Lazy Expiration"

Instead of spawning a complex background timer that drains CPU checking for expired keys, we use a **Lazy Expiration** approach. 

When a `SETEX` command is received, we calculate the exact `SystemTime` it should expire and save it in the DB. Whenever a `GET` command requests data, the `db.get()` first verifies whether the object expired. If expired, we seamlessly delete it right then.

```rust
pub fn get(&self, key: &str) -> Option<String> {
    let mut map = self.store.lock().unwrap();
    let mut should_delete = false;

    if let Some(val) = map.get(key) {
        if let Some(exp) = val.expires_at {
            if SystemTime::now() > exp { should_delete = true; }
        }
    }
    if should_delete { map.remove(key); return None; }
    map.get(key).map(|v| v.data.clone()) 
}
```

### Step 6: Implementing Publish/Subscribe (Pub/Sub)

The server permits clients to subscribe to specific channels via `SUBSCRIBE <channel>`. We store a `.try_clone()` of their `TcpStream` into the database's `pubsub` registry.
When another user broadcasts a message via `PUBLISH <channel> <message>`, we lock the map and broadcast the message down the saved network streams. 

To gracefully handle disconnected subscribers, we rely on the `Vec::retain_mut` iterator trick paired with `write_all`.

```rust
pub fn publish(&self, channel: &str, message: &str) -> usize {
    let mut ps = self.pubsub.lock().unwrap();
    let mut successful_sends = 0;

    if let Some(subscribers) = ps.get_mut(channel) {
        subscribers.retain_mut(|stream| {
            let msg = format!("+MESSAGE {} {}\r\n", channel, message);
            // Try to write. If it fails, return false to remove the zombie stream from the array
            if stream.write_all(msg.as_bytes()).is_ok() {
                successful_sends += 1;
                true 
            } else {
                false 
            }
        });
    }
    successful_sends
}
```

### Step 7: Replying to the Client

Every successful routine answers the client in standard text format. E.g., `SET` returns `+OK\r\n`, and `PUBLISH` returns an integer count of receivers `:2\r\n`.
