mod bmp;

use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::result::Result;

use bmp::MinirtBmp;

fn main() -> Result<(), Box<dyn Error>> {
    // Example usage

    let bmp = MinirtBmp::new(1024, 1024, |x, y| bmp::MinirtBmpPixel {
        r: (x / 4) as u8,
        g: (y / 4) as u8,
        b: 255,
    });
    let serialized = bmp.serialize()?;
    write_bmp_file("example.bmp", &serialized)?;

    let buffer = read_bmp_file("example.bmp")?;
    let bmp = MinirtBmp::deserialize(&buffer)?;
    let serialized = bmp.serialize()?;
    write_bmp_file("output.bmp", &serialized)?;
    Ok(())
}

fn read_bmp_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_bmp_file(filename: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}
