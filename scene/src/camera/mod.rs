use core::types::rt::Camera;
use json::JsonValue;
use perspective::DeserializablePerspectiveCamera;

pub mod perspective;

#[derive(Debug)]
pub enum DeserializableCamera {
    Perspective(DeserializablePerspectiveCamera),
}

impl DeserializableCamera {
    pub fn into_camera(self, screen_aspect_ratio: f64) -> Box<dyn Camera + Send + Sync> {
        match self {
            DeserializableCamera::Perspective(c) => c.into_camera(screen_aspect_ratio),
        }
    }

    pub fn from_json(json: &JsonValue) -> Result<DeserializableCamera, String> {
        return Ok(DeserializableCamera::Perspective(
            DeserializablePerspectiveCamera::from_json(json)?,
        ));
    }
}
