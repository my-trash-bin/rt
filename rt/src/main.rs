use bmp::{MinirtBmp, MinirtBmpPixel};
use scene::{Image, ImageCache, ImageLoader, Scene};
use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::{env, path::PathBuf};
use types::{HDRColor, LDRColor};

use core::types::math::Vec3;

#[derive(Debug)]
struct Args {
    input: String,
    output: Option<String>,
    no_output_bmp_suffix: bool,
    width: Option<usize>,
    height: Option<usize>,
    camera_position: Option<Vec3>,
    camera_direction: Option<Vec3>,
    camera_look_at: Option<Vec3>,
    stdout: bool,
    super_sampling: Option<usize>,
    ambient_light: Option<Vec3>,
    void_color: Option<Vec3>,
    emit_normal: bool,
    emit_distance: bool,
    jobs: Option<usize>,
    gamma: Option<f64>,
    exposure: Option<f64>,
    ldr: bool,
    no_ldr: bool,
}

#[derive(Debug)]
enum ArgsResult {
    Ok(Args),
    Help,
    Version,
}

fn parse_vec3(s: &str, name: &str) -> Result<Vec3, Box<dyn Error>> {
    let parts: Vec<_> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("Expected format x,y,z for {} but got '{}'", name, s).into());
    }
    Ok(Vec3 {
        x: parts[0].parse()?,
        y: parts[1].parse()?,
        z: parts[2].parse()?,
    })
}

fn parse<T: std::str::FromStr>(s: &str, name: &str) -> Result<T, Box<dyn Error>> {
    s.parse()
        .map_err(|_| format!("Invalid value for {}: '{}'", name, s).into())
}

fn args() -> Result<ArgsResult, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    let mut result = Args {
        input: String::new(),
        output: None,
        no_output_bmp_suffix: false,
        width: None,
        height: None,
        camera_position: None,
        camera_direction: None,
        camera_look_at: None,
        stdout: false,
        super_sampling: None,
        ambient_light: None,
        void_color: None,
        emit_normal: false,
        emit_distance: false,
        jobs: None,
        gamma: None,
        exposure: None,
        ldr: false,
        no_ldr: false,
    };
    let mut positionals = vec![];

    while i < args.len() {
        let arg = &args[i];

        if arg == "--help" || arg == "-h" {
            return Ok(ArgsResult::Help);
        }
        if arg == "--version" || arg == "-v" {
            return Ok(ArgsResult::Version);
        }

        if let Some(arg) = arg.strip_prefix("--") {
            let parts: Vec<_> = arg
                .split_once('=')
                .iter()
                .flat_map(|x| vec![x.0, x.1])
                .collect();
            let flag = if parts.len() == 2 { parts[0] } else { arg };
            let value = if parts.len() == 2 {
                Some(parts[1].to_string())
            } else {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    Some(args[i].clone())
                } else {
                    None
                }
            };

            match flag {
                "no-output-bmp-suffix" => result.no_output_bmp_suffix = true,
                "stdout" => {
                    if result.output.is_some() {
                        return Err("--stdout and positional output are mutually exclusive".into());
                    }
                    result.stdout = true;
                }
                _ => return Err(format!("Unknown option --{}", flag).into()),
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            let mut chars = arg[1..].chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    'N' => result.no_output_bmp_suffix = true,
                    'S' => {
                        if result.output.is_some() {
                            return Err("-S and positional output are mutually exclusive".into());
                        }
                        result.stdout = true;
                    }
                    _ => return Err(format!("Unknown short flag: -{}", c).into()),
                }
            }
        } else {
            positionals.push(arg.clone());
        }
        i += 1;
    }

    if positionals.is_empty() {
        return Err("Missing required input file".into());
    }
    result.input = positionals[0].clone();
    if positionals.len() > 1 {
        if result.stdout {
            return Err("Cannot use both output file and --stdout/-S".into());
        }
        result.output = Some(positionals[1].clone());
    } else if !result.stdout {
        return Err("Missing required output file".into());
    }

    Ok(ArgsResult::Ok(result))
}

struct Renderer<'a>(&'a Scene);

impl<'a> Renderer<'a> {
    fn render(&self, x: usize, y: usize) -> MinirtBmpPixel {
        let scene = &self.0 .0;
        let width = scene.image_width as f64;
        let height = scene.image_height as f64;

        let u = (x as f64 + 0.5) / width;
        let v = (y as f64 + 0.5) / height;

        let hdr_color = core::sample(scene, u, v);
        let color = tmp_hdr_to_ldr(hdr_color);

        MinirtBmpPixel {
            r: (color.r * 255.0) as u8,
            g: (color.g * 255.0) as u8,
            b: (color.b * 255.0) as u8,
        }
    }
}

fn main() {
    match args() {
        Ok(ArgsResult::Ok(a)) => {
            if let Err(e) = (|| -> Result<(), String> {
                let json_content = std::fs::read_to_string(&a.input).map_err(|e| e.to_string())?;
                let json_value = jsonc::parse(&json_content)?;

                let image_loader = ImageImageLoader::new(".");
                let mut image_cache = ImageCache::new(&image_loader);
                let scene = Scene::from_json_value(json_value, &mut image_cache)?;

                let r = Renderer(&scene);
                let bmp = MinirtBmp::new(scene.0.image_width, scene.0.image_height, |x, y| {
                    r.render(x, y)
                });

                let bmp_bytes = bmp.serialize();
                if a.stdout {
                    std::io::stdout()
                        .write_all(&bmp_bytes)
                        .map_err(|e| e.to_string())?;
                } else {
                    let output = a.output.unwrap();
                    let output = if output.ends_with(".bmp") || a.no_output_bmp_suffix {
                        output
                    } else {
                        format!("{output}.bmp")
                    };
                    std::fs::write(output, bmp_bytes).map_err(|e| e.to_string())?;
                }
                Ok(())
            })() {
                eprintln!("Error: {}", e);
            }
        }
        Ok(ArgsResult::Help) => println!("Usage: ..."),
        Ok(ArgsResult::Version) => println!("Version 1.0"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn tmp_hdr_to_ldr(color: HDRColor) -> LDRColor {
    const GAMMA: f64 = 2.2;
    const EXPOSURE: f64 = 1.0;

    let r = 1.0 - (-color.r * EXPOSURE).exp();
    let g = 1.0 - (-color.g * EXPOSURE).exp();
    let b = 1.0 - (-color.b * EXPOSURE).exp();

    LDRColor {
        r: r.powf(1.0 / GAMMA),
        g: g.powf(1.0 / GAMMA),
        b: b.powf(1.0 / GAMMA),
    }
}

struct BmpImage {
    image: MinirtBmp,
}

impl BmpImage {
    fn new(path: &str) -> Result<BmpImage, Box<dyn Error>> {
        let buffer = std::fs::read(path)?;
        let image = MinirtBmp::deserialize(&buffer)?;
        Ok(BmpImage { image })
    }
}

impl Image for BmpImage {
    fn width(&self) -> usize {
        self.image.width
    }

    fn height(&self) -> usize {
        self.image.height
    }

    fn get(&self, x: usize, y: usize) -> [f64; 3] {
        if x >= self.width() || y >= self.height() {
            panic!("Incorrect coord given");
        }

        let pixel = &self.image.extra[y * self.image.width + x];

        [
            pixel.r as f64 / 255.0,
            pixel.g as f64 / 255.0,
            pixel.b as f64 / 255.0,
        ]
    }
}

struct ImageImageLoader {
    scene_dir: PathBuf,
}

impl ImageImageLoader {
    fn new<P: AsRef<Path>>(scene_path: P) -> Self {
        let scene_dir = scene_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf();
        ImageImageLoader { scene_dir }
    }
}

impl ImageLoader for ImageImageLoader {
    fn load(&self, path: &str) -> Arc<dyn Image + Send + Sync> {
        let full_path = self.scene_dir.join(path);
        Arc::new(BmpImage::new(full_path.to_str().expect("Invalid path")).expect("Invalid image"))
    }
}
