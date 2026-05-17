// src/db.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use std::net::TcpStream; //   to store network streams
use std::io::Write;      //   write to those streams

#[derive(Debug)]
struct DbValue {
    data: String,
    expires_at: Option<SystemTime>,
}

#[derive(Clone)]
pub struct Database {
    store: Arc<Mutex<HashMap<String, DbValue>>>,
    // 3. New registry: Channel Name -> List of active connections
    pubsub: Arc<Mutex<HashMap<String, Vec<TcpStream>>>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            store: Arc::new(Mutex::new(HashMap::new())),
            pubsub: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /* ... Keep your existing set, set_ex, and get methods exactly as they are ... */
    pub fn set(&self, key: String, value: String) {
        let mut map = self.store.lock().unwrap();
        map.insert(key, DbValue { data: value, expires_at: None });
    }

    pub fn set_ex(&self, key: String, seconds: u64, value: String) {
        let mut map = self.store.lock().unwrap();
        let expiration = SystemTime::now() + Duration::from_secs(seconds);
        map.insert(key, DbValue { data: value, expires_at: Some(expiration) });
    }

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

    // 4. Save a client's stream into the requested channel
    pub fn subscribe(&self, channel: String, stream: TcpStream) {
        let mut ps = self.pubsub.lock().unwrap();
        
        // .entry().or_insert_with() is a brilliant Rust pattern. 
        // If the channel doesn't exist, it creates a new empty Vec.
        let subscribers = ps.entry(channel).or_insert_with(Vec::new);
        subscribers.push(stream);
    }

    // 5. Broadcast a message to everyone listening
    pub fn publish(&self, channel: &str, message: &str) -> usize {
        let mut ps = self.pubsub.lock().unwrap();
        let mut successful_sends = 0;

        if let Some(subscribers) = ps.get_mut(channel) {
            // .retain_mut() loops through the array and keeps only the items that return `true`.
            // This is how we clean up disconnected clients!
            subscribers.retain_mut(|stream| {
                let msg = format!("+MESSAGE {} {}\r\n", channel, message);
                
                // Try to write. If it works, keep the connection. If it fails, drop it.
                if stream.write_all(msg.as_bytes()).is_ok() {
                    successful_sends += 1;
                    true 
                } else {
                    false // Client disconnected, remove them from the array
                }
            });
        }
        
        successful_sends // Return how many people received the message
    }
}