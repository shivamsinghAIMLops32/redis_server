use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// to handle ttl
use std::time::{SystemTime, Duration};

mod parser;
use parser::{Command, parse_command};


// 2. Create a Struct to hold our complex data
#[derive(Debug)]
struct DbValue {
    data: String,
    expires_at: Option<SystemTime>, // "Option" because some keys live forever
}


fn main() {
    //  Create the database state
    // We wrap our HashMap in a Mutex, and then wrap that in an Arc.
    let db = Arc::new(Mutex::new(HashMap::<String, DbValue>::new()));

    // 1. Bind to a port (Open the front door)
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    println!("Mini-Redis is listening on port 6379!");

    // 2. Wait for clients to connect
    for stream in listener.incoming() {
        match stream {
            // making stream mutable because we will need to read from it later
            Ok(mut stream) => {
                println!("A new client connected!");

                //  Clone the Arc pointer (NOT the whole database)
                // This gives this specific thread its own key to access the Mutex
                let db_clone = Arc::clone(&db);

                thread::spawn(move || {
                    loop {
                        // make a bucket to store buffer sent by user in terminal
                        let mut buffer = [0; 512];

                        // 4. Read the data from the stream into our bucket
                        // stream.read gives us the number of bytes read, we will use that to know how much of the buffer is actually filled with data
                        // and the data in buffer filled by stream.read is the command sent by user in terminal
                        match stream.read(&mut buffer) {
                            Ok(size) => {
                                // 3. Handle the client disconnecting
                                if size == 0 {
                                    println!("Client disconnected.");
                                    break;
                                }

                                // 5. Translate the raw bytes into a String
                                let raw_text = String::from_utf8_lossy(&buffer[..size]);
                                // println!("Received command: {}", raw_text);

                                // 3. Use the function from our module to parse the raw_text
                                let command = parse_command(&raw_text);
                                println!("Parsed: {:?}", command);

                                match command {
                                    Command::Ping => {
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                    }
                                    Command::Set(key, value) => {
                                        //  Lock the database, insert the data, let go of the lock
                                        let mut map = db_clone.lock().unwrap();
                                        map.insert(key, DbValue {
                                            data: value,
                                            expires_at: None, 
                                        });
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }

                                    // expiry of a key in db
                                    Command::SetEx(key, seconds, value) => {
                                        let mut map = db_clone.lock().unwrap();
                                        // 4. Calculate exactly when this key should die
                                        let expiration_time = SystemTime::now() + Duration::from_secs(seconds);
                                        
                                        map.insert(key, DbValue {
                                            data: value,
                                            expires_at: Some(expiration_time),
                                        });
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }


                                    Command::Get(key) => {
                                        //  Lock the database, read the data, let go of the lock
                                        let mut map = db_clone.lock().unwrap();
                                        //  Lazy Expiration Logic
                                        let mut should_delete = false;

                                        if let Some(db_value) = map.get(&key) {
                                            if let Some(expiration) = db_value.expires_at {
                                                // Is the current time PAST the expiration time?
                                                if SystemTime::now() > expiration {
                                                    should_delete = true;
                                                }
                                            }
                                        }

                                        if should_delete {
                                            println!("Key '{}' expired! Deleting...", key);
                                            map.remove(&key); // Clean it up!
                                        }

                                        match map.get(&key) {
                                            Some(val) => {
                                                let response = format!("+{}\r\n", val.data);
                                                stream.write_all(response.as_bytes()).unwrap();
                                            }
                                            None => {
                                                stream.write_all(b"$-1\r\n").unwrap(); 
                                            }
                                        };
                                    }
                                    Command::Unknown(cmd) => {
                                        let response =
                                            format!("-ERR unknown command '{}'\r\n", cmd);
                                        stream.write_all(response.as_bytes()).unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to read data: {}", e);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}
