use std::{collections::HashMap, sync::Arc};

use camera::DeserializableCamera;
use core::types::rt::Scene;
use json::JsonValue;
use light::DeserializableLight;
use object::DeserializableRTObject;
use types::HDRColor;

pub mod camera;
pub mod light;
pub mod object;
pub mod texture;

pub trait Image {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: usize, y: usize) -> [f64; 3];
}

pub trait ImageLoader {
    fn load(&self, path: &str) -> Arc<dyn Image + Send + Sync>;
}

pub struct DeserializableScene {
    pub camera: DeserializableCamera,
    pub objects: Vec<DeserializableRTObject>,
    pub lights: Vec<DeserializableLight>,
    pub sky_color: HDRColor,
    pub ambient_light: HDRColor,
}

impl DeserializableScene {
    pub fn into_scene<T: ImageLoader>(self, screen_aspect_ratio: f64, image_loader: &T) -> Scene {
        let mut cache = ImageCache::new(image_loader);
        Scene {
            camera: self.camera.into_camera(screen_aspect_ratio),
            objects: self
                .objects
                .into_iter()
                .map(|o| o.into_rt_object(&mut cache))
                .collect(),
            lights: self
                .lights
                .into_iter()
                .map(DeserializableLight::into_light)
                .collect(),
            sky_color: Arc::new(move |_| self.sky_color),
            ambient_light: self.ambient_light,
        }
    }

    pub fn from_json(json: JsonValue) -> Result<DeserializableScene, String> {
        let dict = match json {
            JsonValue::Dict(dict) => dict,
            _ => return Err("Scene must be a JSON object".to_string()),
        };

        let camera_json_value = dict.get("camera").ok_or("Missing required field: camera")?;
        let camera = DeserializableCamera::from_json(camera_json_value)?;

        let sky_color_json_value = dict
            .get("voidColor")
            .ok_or("Missing required field: voidColor")?;
        let sky_color = match sky_color_json_value {
            JsonValue::List(array) if array.len() == 3 => {
                let r = match &array[0] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("voidColor[0] must be a number".to_string()),
                };
                let g = match &array[1] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("voidColor[1] must be a number".to_string()),
                };
                let b = match &array[2] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("voidColor[2] must be a number".to_string()),
                };
                HDRColor { r, g, b }
            }
            _ => return Err("voidColor must be an array of 3 numbers".to_string()),
        };

        let ambient_light_json_value = dict
            .get("ambientLight")
            .ok_or("Missing required field: ambientLight")?;
        let ambient_light = match ambient_light_json_value {
            JsonValue::List(array) if array.len() == 3 => {
                let r = match &array[0] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("ambientLight[0] must be a number".to_string()),
                };
                let g = match &array[1] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("ambientLight[1] must be a number".to_string()),
                };
                let b = match &array[2] {
                    JsonValue::Number(n) => *n,
                    _ => return Err("ambientLight[2] must be a number".to_string()),
                };
                HDRColor { r, g, b }
            }
            _ => return Err("ambientLight must be an array of 3 numbers".to_string()),
        };

        let mut objects = Vec::new();
        let mut lights = Vec::new();

        if let Some(objects_json) = dict.get("objects") {
            match objects_json {
                JsonValue::List(array) => {
                    for item in array {
                        if let JsonValue::Dict(item_dict) = item {
                            if let Some(JsonValue::String(type_str)) = item_dict.get("type") {
                                match type_str.as_str() {
                                    "point" | "directional" | "spot" => {
                                        // Light 객체
                                        let light = DeserializableLight::from_json(item)?;
                                        lights.push(light);
                                    }
                                    "csg" => {
                                        // CSG 객체
                                        //let object = DeserializableRTObject::from_json(item)?;
                                        //objects.push(object);
                                    }
                                    _ => {
                                        return Err(format!("Unknown object type: {}", type_str));
                                    }
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

        println!("-----DeserializableScene Start-----");
        println!("camera: {:?}", camera);
        println!("sky_color: {:?}", sky_color);
        println!("ambient_light: {:?}", ambient_light);

        println!(
            "Parsed {} objects and {} lights",
            objects.len(),
            lights.len()
        );
        // TODO: CSG 객체
        println!("-----DeserializableScene End-----");

        Ok(DeserializableScene {
            camera,
            objects,
            lights,
            sky_color,
            ambient_light,
        })
    }
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
