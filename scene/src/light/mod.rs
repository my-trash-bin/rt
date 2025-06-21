use core::types::rt::Light;
use directional::DirectionalLight;
use jsonc::Value;
use point::PointLight;

use crate::{
    angle_from_json_value, direction_from_json_value, hdr_color_from_json_value,
    position_from_json_value,
};
pub mod directional;
pub mod point;

/// Parse a light directly from a JSON value.
pub fn from_json_value(json: &Value) -> Result<Box<dyn Light + Send + Sync>, String> {
    let Value::Object(dict) = json else {
        return Err("Light must be a JSON object".to_string());
    };

    let Value::String(type_str) = dict.get("type").ok_or("Missing required field: type")? else {
        return Err("Light must have a 'type' field with string value".to_string());
    };

    let light: Box<dyn Light + Send + Sync> = match type_str.as_str() {
        "point" => {
            let color_json = dict.get("color").ok_or("Missing required field: color")?;
            let color = hdr_color_from_json_value(color_json)?;

            let position_json = dict
                .get("position")
                .ok_or("Missing required field: position")?;
            let position = position_from_json_value(position_json)?;

            // XXX: validation?
            let range = if let Some(Value::Number(r)) = dict.get("range") {
                if *r <= 0.0 {
                    return Err("range must be greater than 0".to_string());
                }
                *r
            } else {
                f64::INFINITY
            };

            // XXX: validation?
            let attenuation = if let Some(Value::Bool(a)) = dict.get("attenuation") {
                *a
            } else {
                true
            };
            Box::new(PointLight::new(color, position, range, attenuation))
        }
        "directional" => {
            let color_json = dict.get("color").ok_or("Missing required field: color")?;
            let color = hdr_color_from_json_value(color_json)?;

            let direction_json = dict
                .get("direction")
                .ok_or("Missing required field: direction")?;
            let direction = direction_from_json_value(direction_json)?;

            Box::new(DirectionalLight::new(color, direction))
        }
        _ => return Err(format!("Unknown light type: {}", type_str)),
    };

    Ok(light)
}
