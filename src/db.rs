// src/db.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use tokio::sync::broadcast;

#[derive(Debug)]
struct DbValue {
    data: String,
    expires_at: Option<SystemTime>,
}

#[derive(Clone)]
pub struct Database {
    store: Arc<Mutex<HashMap<String, DbValue>>>,
    pubsub: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            store: Arc::new(Mutex::new(HashMap::new())),
            pubsub: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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
                if SystemTime::now() > exp { 
                    should_delete = true; 
                }
            }
        }
        
        if should_delete { 
            map.remove(key); 
            return None; 
        }
        
        map.get(key).map(|v| v.data.clone()) 
    }

    pub fn subscribe(&self, channel: String) -> broadcast::Receiver<String> {
        let mut ps = self.pubsub.lock().unwrap();
        
        if let Some(sender) = ps.get(&channel) {
            sender.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(16);
            ps.insert(channel, tx);
            rx
        }
    }

    pub fn publish(&self, channel: &str, message: &str) -> usize {
        let ps = self.pubsub.lock().unwrap();
        
        if let Some(sender) = ps.get(channel) {
            match sender.send(message.to_string()) {
                Ok(count) => count,
                Err(_) => 0, 
            }
        } else {
            0
        }
    }
}