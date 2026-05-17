// src/main.rs

use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

mod parser;
mod db; // 1. Tell Rust about our new db.rs file

use parser::{Command, parse_command};
use db::Database; // 2. Import our Database struct

fn main() {
    // 3. Initialize our clean, abstracted database!
    let db = Database::new(); 

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Mini-Redis is listening on port 6379!");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => { 
                // 4. Clone the database reference for this thread
                let db_clone = db.clone(); 

                thread::spawn(move || {
                    loop {
                        let mut buffer = [0; 512]; 
                        match stream.read(&mut buffer) {
                            Ok(size) => {
                                if size == 0 { break; }

                                let raw_text = String::from_utf8_lossy(&buffer[..size]);
                                let command = parse_command(&raw_text);

                                match command {
                                    Command::Ping => {
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                    }
                                    Command::Set(key, value) => {
                                        // 5. Look how clean this is!
                                        db_clone.set(key, value); 
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }
                                    Command::SetEx(key, seconds, value) => {
                                        db_clone.set_ex(key, seconds, value);
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }
                                    Command::Get(key) => {
                                        // 6. Handle the Option returned by our DB
                                        match db_clone.get(&key) {
                                            Some(val) => {
                                                let response = format!("+{}\r\n", val);
                                                stream.write_all(response.as_bytes()).unwrap();
                                            }
                                            None => {
                                                stream.write_all(b"$-1\r\n").unwrap(); 
                                            }
                                        }
                                    }
                                    Command::Unknown(cmd) => {
                                        let response = format!("-ERR unknown command '{}'\r\n", cmd);
                                        stream.write_all(response.as_bytes()).unwrap();
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
            Err(e) => println!("Failed to connect: {}", e),
        }
    }
}