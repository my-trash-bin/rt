use scene::{Image, ImageLoader, Scene};
use std::env;
use std::error::Error;
use std::sync::Arc;

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
            let flag = parts[0];
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
                "width" => {
                    result.width = Some(parse(
                        value.ok_or("Missing --width value")?.as_str(),
                        "width",
                    )?)
                }
                "height" => {
                    result.height = Some(parse(
                        value.ok_or("Missing --height value")?.as_str(),
                        "height",
                    )?)
                }
                "camera-position" => {
                    result.camera_position = Some(parse_vec3(
                        value.ok_or("Missing --camera-position")?.as_str(),
                        "camera-position",
                    )?)
                }
                "camera-direction" => {
                    if result.camera_look_at.is_some() {
                        return Err(
                            "--camera-direction and --camera-look-at are mutually exclusive".into(),
                        );
                    }
                    result.camera_direction = Some(parse_vec3(
                        value.ok_or("Missing --camera-direction")?.as_str(),
                        "camera-direction",
                    )?)
                }
                "camera-look-at" => {
                    if result.camera_direction.is_some() {
                        return Err(
                            "--camera-look-at and --camera-direction are mutually exclusive".into(),
                        );
                    }
                    result.camera_look_at = Some(parse_vec3(
                        value.ok_or("Missing --camera-look-at")?.as_str(),
                        "camera-look-at",
                    )?)
                }
                "stdout" => {
                    if result.output.is_some() {
                        return Err("--stdout and positional output are mutually exclusive".into());
                    }
                    result.stdout = true;
                }
                "super-sampling" => {
                    result.super_sampling = Some(parse(
                        value.ok_or("Missing --super-sampling")?.as_str(),
                        "super-sampling",
                    )?)
                }
                "ambient-light" => {
                    result.ambient_light = Some(parse_vec3(
                        value.ok_or("Missing --ambient-light")?.as_str(),
                        "ambient-light",
                    )?)
                }
                "void-color" => {
                    result.void_color = Some(parse_vec3(
                        value.ok_or("Missing --void-color")?.as_str(),
                        "void-color",
                    )?)
                }
                "emit-normal" => result.emit_normal = true,
                "emit-distance" => result.emit_distance = true,
                "jobs" => {
                    result.jobs = Some(parse(value.ok_or("Missing --jobs")?.as_str(), "jobs")?)
                }
                "gamma" => {
                    if result.ldr {
                        return Err("--gamma and --ldr are mutually exclusive".into());
                    }
                    result.gamma = Some(parse(value.ok_or("Missing --gamma")?.as_str(), "gamma")?)
                }
                "exposure" => {
                    if result.ldr {
                        return Err("--exposure and --ldr are mutually exclusive".into());
                    }
                    result.exposure = Some(parse(
                        value.ok_or("Missing --exposure")?.as_str(),
                        "exposure",
                    )?)
                }
                "ldr" => {
                    if result.gamma.is_some() || result.exposure.is_some() || result.no_ldr {
                        return Err(
                            "--ldr and --gamma/--exposure/--no-ldr are mutually exclusive".into(),
                        );
                    }
                    result.ldr = true;
                }
                "no-ldr" => {
                    if result.ldr {
                        return Err("--no-ldr and --ldr are mutually exclusive".into());
                    }
                    result.no_ldr = true;
                }
                _ => return Err(format!("Unknown option --{}", flag).into()),
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            let mut chars = arg[1..].chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    'N' => result.no_output_bmp_suffix = true,
                    'n' => result.emit_normal = true,
                    'd' => result.emit_distance = true,
                    'S' => {
                        if result.output.is_some() {
                            return Err("-S and positional output are mutually exclusive".into());
                        }
                        result.stdout = true;
                    }
                    'W' | 'H' | 's' | 'a' | 'v' | 'j' | 'g' | 'e' | 'P' | 'D' | 'L' => {
                        let mut val: String = chars.collect();
                        if val.is_empty() {
                            i += 1;
                            if i >= args.len() {
                                return Err(format!("Missing value for -{}", c).into());
                            }
                            val = args[i].clone();
                        }
                        match c {
                            'W' => result.width = Some(parse(&val, "-W")?),
                            'H' => result.height = Some(parse(&val, "-H")?),
                            's' => result.super_sampling = Some(parse(&val, "-s")?),
                            'a' => result.ambient_light = Some(parse_vec3(&val, "-a")?),
                            'v' => result.void_color = Some(parse_vec3(&val, "-v")?),
                            'j' => result.jobs = Some(parse(&val, "-j")?),
                            'g' => {
                                if result.ldr {
                                    return Err("-g and -l are mutually exclusive".into());
                                }
                                result.gamma = Some(parse(&val, "-g")?)
                            }
                            'e' => {
                                if result.ldr {
                                    return Err("-e and -l are mutually exclusive".into());
                                }
                                result.exposure = Some(parse(&val, "-e")?)
                            }
                            'P' => result.camera_position = Some(parse_vec3(&val, "-P")?),
                            'D' => {
                                if result.camera_look_at.is_some() {
                                    return Err("-D and -L are mutually exclusive".into());
                                }
                                result.camera_direction = Some(parse_vec3(&val, "-D")?)
                            }
                            'L' => {
                                if result.camera_direction.is_some() {
                                    return Err("-L and -D are mutually exclusive".into());
                                }
                                result.camera_look_at = Some(parse_vec3(&val, "-L")?)
                            }
                            _ => {}
                        }
                        break;
                    }
                    'l' => {
                        if result.gamma.is_some() || result.exposure.is_some() {
                            return Err("-l and -g/-e are mutually exclusive".into());
                        }
                        result.ldr = true;
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
    }

    Ok(ArgsResult::Ok(result))
}

fn main() {
    println!("Program entry point");
    match args() {
        Ok(ArgsResult::Ok(a)) => {
            if let Err(e) = (|| -> Result<(), String> {
                let json_content = std::fs::read_to_string(&a.input).map_err(|e| e.to_string())?;
                let json_value = jsonc::parse(&json_content)?;
                struct DummyImage;
                impl Image for DummyImage {
                    fn width(&self) -> usize {
                        1
                    }
                    fn height(&self) -> usize {
                        1
                    }
                    fn get(&self, _x: usize, _y: usize) -> [f64; 3] {
                        [0.0, 0.0, 0.0]
                    }
                }

                struct DummyLoader;
                impl ImageLoader for DummyLoader {
                    fn load(&self, _path: &str) -> Arc<dyn Image + Send + Sync> {
                        Arc::new(DummyImage)
                    }
                }

                let loader = DummyLoader;
                let _scene = Scene::from_json_value(json_value, 1.0, &loader)?;
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
