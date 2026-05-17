// src/bin/benchmark.rs

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Instant;

fn main() {
    let num_threads = 50;
    let requests_per_thread = 1_000;
    let total_requests = num_threads * requests_per_thread;

    println!("Starting benchmark...");
    println!("Simulating {} concurrent users...", num_threads);

    let start_time = Instant::now();
    let mut handles = vec![];

    // Spawn 50 concurrent users
    for _ in 0..num_threads {
        let handle = thread::spawn(move || {
            // Each user connects to the database
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:6379") {
                let mut buffer = [0; 512];
                
                // Each user fires off 1,000 requests
                for _ in 0..requests_per_thread {
                    stream.write_all(b"PING\r\n").unwrap();
                    let _ = stream.read(&mut buffer); // Wait for the PONG
                }
            } else {
                println!("Failed to connect. Is the server running?");
            }
        });
        handles.push(handle);
    }

    // Wait for all 50 users to finish their 1,000 requests
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