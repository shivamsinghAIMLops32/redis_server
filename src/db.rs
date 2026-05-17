use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

// 1. Keep the internal data structure private to this file
#[derive(Debug)]
struct DbValue {
    data: String,
    expires_at: Option<SystemTime>,
}

// 2. The public Database struct that our server will use
#[derive(Clone)] // This allows us to cheaply clone the Arc pointer!
pub struct Database {
    store: Arc<Mutex<HashMap<String, DbValue>>>,
}

// 3. The Implementation block (like a Class in other languages)
impl Database {
    // A constructor to create a new database
    pub fn new() -> Self {
        Database {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Method to handle standard SET
    pub fn set(&self, key: String, value: String) {
        let mut map = self.store.lock().unwrap();
        map.insert(key, DbValue { data: value, expires_at: None });
    }

    // Method to handle SETEX
    pub fn set_ex(&self, key: String, seconds: u64, value: String) {
        let mut map = self.store.lock().unwrap();
        let expiration = SystemTime::now() + Duration::from_secs(seconds);
        map.insert(key, DbValue { data: value, expires_at: Some(expiration) });
    }

    // Method to handle GET and lazy expiration
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
            return None; // Return nothing if it expired
        }

        // Return a clone of the string so we don't accidentally hold 
        // the Mutex lock longer than we need to!
        map.get(key).map(|v| v.data.clone()) 
    }
}