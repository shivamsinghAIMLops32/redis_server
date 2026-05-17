// src/main.rs

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod parser;
mod db;

use parser::{Command, parse_command};
use db::Database;

#[tokio::main]
async fn main() {
    let db = Database::new(); 
    
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Mini-Redis (Async/Tokio) is listening on port 6379!");

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                stream.set_nodelay(true).unwrap();
                
                let db_clone = db.clone(); 
                
                tokio::spawn(async move {
                    loop {
                        let mut buffer = [0; 512]; 
                        
                        match stream.read(&mut buffer).await {
                            Ok(size) => {
                                if size == 0 { break; }

                                let raw_text = String::from_utf8_lossy(&buffer[..size]);
                                let command = parse_command(&raw_text);

                                match command {
                                    Command::Ping => {
                                        stream.write_all(b"+PONG\r\n").await.unwrap();
                                    }
                                    Command::Set(key, value) => {
                                        db_clone.set(key, value); 
                                        stream.write_all(b"+OK\r\n").await.unwrap();
                                    }
                                    Command::SetEx(key, seconds, value) => {
                                        db_clone.set_ex(key, seconds, value);
                                        stream.write_all(b"+OK\r\n").await.unwrap();
                                    }
                                    Command::Get(key) => {
                                        match db_clone.get(&key) {
                                            Some(val) => {
                                                let response = format!("+{}\r\n", val);
                                                stream.write_all(response.as_bytes()).await.unwrap();
                                            }
                                            None => {
                                                stream.write_all(b"$-1\r\n").await.unwrap(); 
                                            }
                                        }
                                    }
                                    Command::Subscribe(channel) => {
                                        let mut rx = db_clone.subscribe(channel.clone());
                                        
                                        let response = format!("+SUBSCRIBED to {}\r\n", channel);
                                        stream.write_all(response.as_bytes()).await.unwrap();

                                        loop {
                                            match rx.recv().await {
                                                Ok(msg) => {
                                                    let response = format!("+MESSAGE {} {}\r\n", channel, msg);
                                                    if stream.write_all(response.as_bytes()).await.is_err() {
                                                        break; 
                                                    }
                                                }
                                                Err(_) => break, 
                                            }
                                        }
                                        break; 
                                    }
                                    Command::Publish(channel, msg) => {
                                        let count = db_clone.publish(&channel, &msg);
                                        let response = format!(":{}\r\n", count);
                                        stream.write_all(response.as_bytes()).await.unwrap();
                                    }
                                    Command::Unknown(cmd) => {
                                        let response = format!("-ERR unknown command '{}'\r\n", cmd);
                                        stream.write_all(response.as_bytes()).await.unwrap();
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
            Err(e) => println!("Failed to accept connection: {}", e),
        }
    }
}