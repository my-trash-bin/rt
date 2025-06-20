use std::{collections::HashMap, sync::Arc};

use core::types::rt::Scene as CoreScene;
use jsonc::Value;
use object::DeserializableRTObject; // objects still use builder
use types::HDRColor;

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
        screen_aspect_ratio: f64,
        image_loader: &T,
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

        let camera_json = dict.get("camera").ok_or("Missing required field: camera")?;
        let camera = camera::from_json_value(camera_json, screen_aspect_ratio)?;

        let sky_color_json = dict
            .get("voidColor")
            .ok_or("Missing required field: voidColor")?;
        let sky_color = match sky_color_json {
            Value::Array(array) if array.len() == 3 => {
                let r = match &array[0] {
                    Value::Number(n) => *n,
                    _ => return Err("voidColor[0] must be a number".to_string()),
                };
                let g = match &array[1] {
                    Value::Number(n) => *n,
                    _ => return Err("voidColor[1] must be a number".to_string()),
                };
                let b = match &array[2] {
                    Value::Number(n) => *n,
                    _ => return Err("voidColor[2] must be a number".to_string()),
                };
                HDRColor { r, g, b }
            }
            _ => return Err("voidColor must be an array of 3 numbers".to_string()),
        };

        let ambient_light_json = dict
            .get("ambientLight")
            .ok_or("Missing required field: ambientLight")?;
        let ambient_light = match ambient_light_json {
            Value::Array(array) if array.len() == 3 => {
                let r = match &array[0] {
                    Value::Number(n) => *n,
                    _ => return Err("ambientLight[0] must be a number".to_string()),
                };
                let g = match &array[1] {
                    Value::Number(n) => *n,
                    _ => return Err("ambientLight[1] must be a number".to_string()),
                };
                let b = match &array[2] {
                    Value::Number(n) => *n,
                    _ => return Err("ambientLight[2] must be a number".to_string()),
                };
                HDRColor { r, g, b }
            }
            _ => return Err("ambientLight must be an array of 3 numbers".to_string()),
        };

        let mut objects: Vec<DeserializableRTObject> = Vec::new();
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
                                        //let object = DeserializableRTObject::from_json(item)?;
                                        //objects.push(object);
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

        let mut cache = ImageCache::new(image_loader);

        Ok(Scene(CoreScene {
            image_width,
            image_height,
            camera,
            objects: objects
                .into_iter()
                .map(|o| o.into_rt_object(&mut cache))
                .collect(),
            lights,
            sky_color: Arc::new(move |_| sky_color),
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
