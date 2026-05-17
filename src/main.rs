use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
fn main() {
    // 1. Bind to a port (Open the front door)
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    println!("Mini-Redis is listening on port 6379!");

    // 2. Wait for clients to connect
    for stream in listener.incoming() {
        match stream {
            // making stream mutable because we will need to read from it later
            Ok(mut stream) => {
                println!("A new client connected!");

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
                                let command = String::from_utf8_lossy(&buffer[..size]);
                                println!("Received command: {}", command);

                                // 4. Send a response back!
                                stream.write_all(b"+PONG\r\n").unwrap();
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
