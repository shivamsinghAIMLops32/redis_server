use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod parser;
use parser::{Command, parse_command};

fn main() {
    //  Create the database state
    // We wrap our HashMap in a Mutex, and then wrap that in an Arc.
    let db = Arc::new(Mutex::new(HashMap::new()));

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
                                        map.insert(key, value);
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }
                                    Command::Get(key) => {
                                        //  Lock the database, read the data, let go of the lock
                                        let map = db_clone.lock().unwrap();
                                        match map.get(&key) {
                                            Some(val) => {
                                                // Format it the way Redis expects for text
                                                let response = format!("+{}\r\n", val);
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
