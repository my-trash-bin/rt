use std::{collections::HashMap, sync::Arc};

use core::types::{
    math::{Direction, Position, Vec3},
    rt::{RTObject, Scene as CoreScene},
};
use jsonc::Value;
use types::{HDRColor, LDRColor};

pub mod camera;
pub mod light;
pub mod object;
pub mod texture;

pub struct Scene(pub CoreScene);

impl From<CoreScene> for Scene {
    fn from(scene: CoreScene) -> Self {
        Scene(scene)
    }
}

impl From<Scene> for CoreScene {
    fn from(scene: Scene) -> Self {
        scene.0
    }
}

impl Scene {
    pub fn from_json_value<T: ImageLoader>(
        json: Value,
        image_cache: &mut ImageCache<T>,
    ) -> Result<Self, String> {
        let dict = match json {
            Value::Object(dict) => dict,
            _ => return Err("Scene must be a JSON object".to_string()),
        };

        // deserialize imageSize from json
        let image_size = dict
            .get("imageSize")
            .ok_or("Missing required field: imageSize")?;
        let (image_width, image_height) = match image_size {
            Value::Object(dict) => {
                let width = match dict
                    .get("width")
                    .ok_or("Missing required field: image width")?
                {
                    Value::Number(w) => *w,
                    _ => return Err("image width must be a number".to_string()),
                };
                let height = match dict
                    .get("height")
                    .ok_or("Missing required field: image height")?
                {
                    Value::Number(h) => *h,
                    _ => return Err("image height must be a number".to_string()),
                };
                (width as usize, height as usize)
            }
            _ => return Err("imageSize must be a JSON object".to_string()),
        };

        let screen_aspect_ratio = image_width as f64 / image_height as f64;
        let camera_json = dict.get("camera").ok_or("Missing required field: camera")?;
        let camera = camera::from_json_value(camera_json, screen_aspect_ratio)?;

        let void_color = hdr_color_from_json_value(
            dict.get("voidColor")
                .ok_or("Missing required field: voidColor")?,
        )?;

        let ambient_light = hdr_color_from_json_value(
            dict.get("ambientLight")
                .ok_or("Missing required field: ambientLight")?,
        )?;

        let mut objects: Vec<Box<dyn RTObject + Send + Sync>> = Vec::new();
        let mut lights: Vec<Box<dyn core::types::rt::Light + Send + Sync>> = Vec::new();

        if let Some(objects_json) = dict.get("objects") {
            match objects_json {
                Value::Array(array) => {
                    for item in array {
                        if let Value::Object(item_dict) = item {
                            if let Some(Value::String(type_str)) = item_dict.get("type") {
                                match type_str.as_str() {
                                    "point" | "directional" | "spot" => {
                                        let light = light::from_json_value(item)?;
                                        lights.push(light);
                                    }
                                    "csg" => {
                                        let model = item_dict
                                            .get("model")
                                            .ok_or("Missing required field: model")?;
                                        let object = object::from_json_value(model, image_cache)?;
                                        objects.push(object);
                                    }
                                    _ => return Err(format!("Unknown object type: {}", type_str)),
                                }
                            } else {
                                return Err("Object must have a 'type' field".to_string());
                            }
                        } else {
                            return Err("Object must be a JSON object".to_string());
                        }
                    }
                }
                _ => return Err("objects must be an array".to_string()),
            }
        }

        Ok(Scene(CoreScene {
            image_width,
            image_height,
            camera,
            objects,
            lights,
            sky_color: Arc::new(move |_| void_color),
            ambient_light,
        }))
    }
}

pub trait Image {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: usize, y: usize) -> [f64; 3];
}

pub trait ImageLoader {
    fn load(&self, path: &str) -> Arc<dyn Image + Send + Sync>;
}

pub struct ImageCache<'a, T: ImageLoader> {
    loader: &'a T,
    cache: HashMap<String, Arc<dyn Image + Send + Sync>>,
}

impl<'a, T: ImageLoader> ImageCache<'a, T> {
    pub fn new(loader: &'a T) -> ImageCache<'a, T> {
        ImageCache {
            loader,
            cache: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: &str) -> Arc<dyn Image + Send + Sync> {
        if let Some(image) = self.cache.get(path) {
            return image.clone();
        }

        let loaded_image: Arc<dyn Image + Send + Sync> = self.loader.load(path);
        self.cache.insert(path.to_string(), loaded_image.clone());

        loaded_image
    }
}

pub fn ldr_color_from_json_value(json: &Value) -> Result<LDRColor, String> {
    let Value::Array(array) = json else {
        return Err("Color must be an array of 3 numbers".to_string());
    };
    if array.len() != 3 {
        return Err("Color must be an array of 3 numbers".to_string());
    }
    let r = match &array[0] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    let g = match &array[1] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    let b = match &array[2] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    if r < 0.0 || r > 1.0 || g < 0.0 || g > 1.0 || b < 0.0 || b > 1.0 {
        return Err("Color must be an array of 3 numbers between 0 and 1".to_string());
    }
    Ok(LDRColor { r, g, b })
}

pub fn hdr_color_from_json_value(json: &Value) -> Result<HDRColor, String> {
    let Value::Array(array) = json else {
        return Err("Color must be an array of 3 numbers".to_string());
    };
    if array.len() != 3 {
        return Err("Color must be an array of 3 numbers".to_string());
    }
    let r = match &array[0] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    let g = match &array[1] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    let b = match &array[2] {
        Value::Number(n) => *n,
        _ => return Err("Color must be an array of 3 numbers".to_string()),
    };
    if r < 0.0 || g < 0.0 || b < 0.0 {
        return Err("Color must be an array of 3 numbers greater than 0".to_string());
    }
    Ok(HDRColor { r, g, b })
}

fn position_from_json_value(json: &Value) -> Result<Position, String> {
    let Value::Array(array) = json else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    if array.len() != 3 {
        return Err("Position must be an array of 3 numbers".to_string());
    }
    let Value::Number(x) = &array[0] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(y) = &array[1] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(z) = &array[2] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    Ok(Position::new(Vec3::new(*x, *y, *z)))
}

fn direction_from_json_value(json: &Value) -> Result<Direction, String> {
    let Value::Array(array) = json else {
        return Err("Direction must be an array of 3 numbers".to_string());
    };
    if array.len() != 3 {
        return Err("Position must be an array of 3 numbers".to_string());
    }
    let Value::Number(x) = &array[0] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(y) = &array[1] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(z) = &array[2] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let vec = Vec3::new(*x, *y, *z);
    if vec.length() < 1e-10 {
        return Err("direction vector cannot be zero or near-zero".to_string());
    }

    Ok(Direction::new(vec))
}

fn angle_from_json_value(json: &Value) -> Result<f64, String> {
    let Value::Object(dict) = json else {
        return Err("angle must be a JSON object".to_string());
    };

    if let Some(Value::Number(degree)) = dict.get("degree") {
        Ok(degree.to_radians())
    } else if let Some(Value::Number(radian)) = dict.get("radian") {
        Ok(*radian)
    } else if let Some(Value::Number(rotation)) = dict.get("rotation") {
        Ok((rotation * 360.0).to_radians())
    } else {
        Err("angle must have either 'degree' or 'radian' or 'rotation' field".to_string())
    }
}

fn scale_from_json_value(json: &Value) -> Result<Vec3, String> {
    let Value::Array(array) = json else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    if array.len() != 3 {
        return Err("Position must be an array of 3 numbers".to_string());
    }
    let Value::Number(x) = &array[0] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(y) = &array[1] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    let Value::Number(z) = &array[2] else {
        return Err("Position must be an array of 3 numbers".to_string());
    };
    Ok(Vec3::new(*x, *y, *z))
}
