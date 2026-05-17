// We add "pub" so main.rs is allowed to see this enum
#[derive(Debug)]
pub enum Command {
    Ping,
    Set(String, String),
    SetEx(String, u64, String), // Key, Seconds, Value
    Get(String),
    Unknown(String),
}

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
        _ => Command::Unknown(cmd),
    }
}
