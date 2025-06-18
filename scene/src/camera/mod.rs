use core::types::rt::Camera;
use jsonc::Value;
use perspective as persp;

pub mod perspective;

/// Parse a camera directly from a JSON value.
pub fn from_json_value(
    json: &Value,
    screen_aspect_ratio: f64,
) -> Result<Box<dyn Camera + Send + Sync>, String> {
    persp::from_json_value(json, screen_aspect_ratio)
}
