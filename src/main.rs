use std::net::TcpListener;

fn main() {
    // 1. Bind to a port (Open the front door)
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    println!("Mini-Redis is listening on port 6379!");

    // 2. Wait for clients to connect
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("A new client connected!");
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}