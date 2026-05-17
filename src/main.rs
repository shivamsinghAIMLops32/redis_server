use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

mod parser;
mod db;

use parser::{Command, parse_command};
use db::Database;

fn main() {
    let db = Database::new(); 
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Mini-Redis is listening on port 6379!");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => { 
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
                                        db_clone.set(key, value); 
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }
                                    Command::SetEx(key, seconds, value) => {
                                        db_clone.set_ex(key, seconds, value);
                                        stream.write_all(b"+OK\r\n").unwrap();
                                    }
                                    Command::Get(key) => {
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
                                    // --- NEW PUB/SUB LOGIC ---
                                    Command::Subscribe(channel) => {
                                        // 1. Clone the socket handle safely
                                        let stream_clone = stream.try_clone().expect("Failed to clone stream");
                                        // 2. Hand the clone over to the database to store
                                        db_clone.subscribe(channel.clone(), stream_clone);
                                        
                                        let response = format!("+SUBSCRIBED to {}\r\n", channel);
                                        stream.write_all(response.as_bytes()).unwrap();
                                    }
                                    Command::Publish(channel, msg) => {
                                        // 3. Broadcast the message and find out how many heard it
                                        let count = db_clone.publish(&channel, &msg);
                                        
                                        // Redis replies with the integer count of receivers (e.g., ":2\r\n")
                                        let response = format!(":{}\r\n", count);
                                        stream.write_all(response.as_bytes()).unwrap();
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