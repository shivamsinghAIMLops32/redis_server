// src/parser.rs

#[derive(Debug)]
pub enum Command {
    Ping,
    Set(String, String),
    SetEx(String, u64, String),
    Get(String),
    Subscribe(String),
    Publish(String, String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
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
            let message = parts.collect::<Vec<&str>>().join(" ");
            Command::Publish(channel, message)
        }
        _ => Command::Unknown(cmd),
    }
}