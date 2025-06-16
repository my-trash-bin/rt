use core::types::rt::Camera;
use perspective::DeserializablePerspectiveCamera;

pub mod perspective;

pub enum DeserializableCamera {
    Perspective(DeserializablePerspectiveCamera),
}

impl DeserializableCamera {
    pub fn into_camera(self, screen_aspect_ratio: f64) -> Box<dyn Camera + Send + Sync> {
        match self {
            DeserializableCamera::Perspective(c) => c.into_camera(screen_aspect_ratio),
        }
    }
}
