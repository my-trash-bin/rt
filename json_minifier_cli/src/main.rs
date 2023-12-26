use std::io::{self, Read};

use json::JsonValue;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");

    let json = JsonValue::new(input.as_str());
    match json {
        Err(reason) => panic!("Failed to parse input: {}", reason),
        Ok(json) => println!("{}", json.serialize()),
    }
}
