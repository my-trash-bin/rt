use std::error::Error;
use std::result::Result;

#[derive(Debug, PartialEq)]
pub struct MinirtBmpPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq)]
pub struct MinirtBmp {
    width: usize,
    height: usize,
    extra: Vec<MinirtBmpPixel>,
}

impl MinirtBmp {
    pub fn new(width: usize, height: usize, fill: fn(usize, usize) -> MinirtBmpPixel) -> MinirtBmp {
        let mut extra = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                extra.push(fill(x, y));
            }
        }
        MinirtBmp {
            width,
            height,
            extra,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let row_padding = (4 - (self.width * 3) % 4) % 4;
        let row_size = self.width * 3 + row_padding;
        let whole_size = row_size * self.height;
        let mut result = Vec::with_capacity(54 + whole_size);

        // Fill header
        result.extend(b"BM");
        result.extend(&(54 + whole_size as u32).to_le_bytes());
        result.extend(&[0, 0, 0, 0]);
        result.extend(&54u32.to_le_bytes());
        result.extend(&40u32.to_le_bytes());
        result.extend(&(self.width as u32).to_le_bytes());
        result.extend(&(self.height as u32).to_le_bytes());
        result.extend(&1u16.to_le_bytes());
        result.extend(&24u16.to_le_bytes());
        result.extend(&[0, 0, 0, 0]);
        result.extend(&(whole_size as u32).to_le_bytes());
        result.extend(&[0, 0, 0, 0]);
        result.extend(&[0, 0, 0, 0]);
        result.extend(&24u32.to_le_bytes());
        result.extend(&[0, 0, 0, 0]);

        // Fill body
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let pixel = &self.extra[y * self.width + x];
                result.push(pixel.b);
                result.push(pixel.g);
                result.push(pixel.r);
            }
            for _ in 0..row_padding {
                result.push(0);
            }
        }

        Ok(result)
    }

    pub fn deserialize(buffer: &[u8]) -> Result<MinirtBmp, Box<dyn Error>> {
        if buffer.len() <= 54 || &buffer[0..2] != b"BM" {
            return Err("Invalid BMP file format".into());
        }

        let width = i32::abs(i32::from_le_bytes(buffer[18..22].try_into()?)) as usize;
        let height = i32::abs(i32::from_le_bytes(buffer[22..26].try_into()?)) as usize;
        let row_padding = (4 - (width * 3) % 4) % 4;
        let row_size = width * 3 + row_padding;
        let whole_size = row_size * height - row_padding;

        if buffer.len() < 54 + whole_size {
            return Err("Invalid BMP file size".into());
        }

        let mut extra = Vec::with_capacity(width * height);
        for y in (0..height).rev() {
            for x in 0..width {
                let r = buffer[54 + y * row_size + x * 3 + 2];
                let g = buffer[54 + y * row_size + x * 3 + 1];
                let b = buffer[54 + y * row_size + x * 3];
                extra.push(MinirtBmpPixel { r, g, b });
            }
        }

        Ok(MinirtBmp {
            width,
            height,
            extra,
        })
    }
}
