first we create a port binding so tcplistener can listen and user can connect to a port

second we we loop over streams send by tcp connection

third we handled error and ok condition return by stream

then we create a buffer then passed it into stream.read(&mut buffer) to get the content user sent into buffer and also the size of buffer user sent 

let command = String::from_utf8_lossy(&buffer[..size]);
                                println!("Received command: {}", command

then we write a respones back to user using stream.writeall in binary not string
then we had a continuous loop to keep litening each commands
then we created thread to handle concurrent connection from multiple users

wrote a basic parser function and enum to handle user input

now we can create our hashmap inmemeory db but use arc and mutex to safely share it in many threads