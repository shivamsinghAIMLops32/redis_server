// src/bin/benchmark.rs

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Instant;

fn main() {
    let num_threads = 500;
    let requests_per_thread = 4_000;
    let total_requests = num_threads * requests_per_thread;

    println!("Starting benchmark...");
    println!("Simulating {} concurrent users...", num_threads);

    let start_time = Instant::now();
    let mut handles = vec![];

    for _ in 0..num_threads {
        let handle = thread::spawn(move || {
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:6379") {
                stream.set_nodelay(true).unwrap();
                let mut buffer = [0; 512];
                
                for _ in 0..requests_per_thread {
                    stream.write_all(b"PING\r\n").unwrap();
                    let _ = stream.read(&mut buffer); 
                }
            } else {
                println!("Failed to connect. Is the server running?");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start_time.elapsed();
    let seconds = elapsed.as_secs_f64();
    let req_per_sec = total_requests as f64 / seconds;

    println!("--- Benchmark Complete ---");
    println!("Total Requests: {}", total_requests);
    println!("Time Elapsed:   {:.2} seconds", seconds);
    println!("Requests/Sec:   {:.2} RPS", req_per_sec);
}