use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file1> <file2> ...", &args[0]);
        process::exit(1);
    }

    let mut input_map = HashMap::new();

    for file_path in &args[1..] {
        match read_file(file_path) {
            Ok((key, content)) => {
                input_map.insert(key, content);
            }
            Err(err) => {
                eprintln!("Error reading {}: {}", file_path, err);
                process::exit(1);
            }
        }
    }

    let packed_data = pack::pack(input_map);

    // Print the packed data to stdout
    io::stdout()
        .write_all(&packed_data)
        .expect("Failed to write to stdout");
}

fn read_file(file_path: &str) -> Result<(String, Vec<u8>), io::Error> {
    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    Ok((file_path.to_string(), content))
}
