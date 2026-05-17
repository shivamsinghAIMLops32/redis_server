#[derive(Debug)]
pub enum Command {
    Ping,
    Set(String, String),
    SetEx(String, u64, String),
    Get(String),
    // Make sure these two lines are here!
    Subscribe(String),
    Publish(String, String),
    Unknown(String),
}

// ... rest of your parse_command function ...

// We add "pub" so main.rs is allowed to call this function
pub fn parse_command(input: &str) -> Command {
    //.split_whitespace() creates an Iterator. Instead of loading everything into an array at once, it yields the next word every time we call .next(). If there are no more words, .next() returns None
    let mut parts = input.trim().split_whitespace();
    let cmd = parts.next().unwrap_or("").to_uppercase();

    match cmd.as_str() {
        "PING" => Command::Ping,
        "SET" => {
            let key = parts.next().unwrap_or("").to_string();
            let value = parts.next().unwrap_or("").to_string();
            Command::Set(key, value)
        }
        "SETEX" => {
            let key = parts.next().unwrap_or("").to_string();
            // Parse the seconds into a number, default to 0 if they type text by mistake
            let seconds = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
            let value = parts.next().unwrap_or("").to_string();
            Command::SetEx(key, seconds, value)
        }
        "GET" => {
            let key = parts.next().unwrap_or("").to_string();
            Command::Get(key)
        }
        "SUBSCRIBE" => {
            let channel = parts.next().unwrap_or("").to_string();
            Command::Subscribe(channel)
        }
        "PUBLISH" => {
            let channel = parts.next().unwrap_or("").to_string();
            // Collect the rest of the words as the message
            let message = parts.collect::<Vec<&str>>().join(" ");
            Command::Publish(channel, message)
        }
        _ => Command::Unknown(cmd),
    }
}
