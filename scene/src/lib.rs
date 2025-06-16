use std::{collections::HashMap, sync::Arc};

use camera::DeserializableCamera;
use core::types::rt::Scene;
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
