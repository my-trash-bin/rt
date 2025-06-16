use core::types::rt::Light;
use directional::DirectionalLight;
use point::PointLight;

pub mod directional;
pub mod point;

pub enum DeserializableLight {
    Point(PointLight),
    Directional(DirectionalLight),
}

impl DeserializableLight {
    pub fn into_light(self) -> Box<dyn Light + Send + Sync> {
        match self {
            DeserializableLight::Point(c) => Box::new(c),
            DeserializableLight::Directional(c) => Box::new(c),
        }
    }
}
