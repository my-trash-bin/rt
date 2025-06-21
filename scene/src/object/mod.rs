use core::types::rt::RTObject;
use jsonc::Value;
use types::LDRColor;

use crate::{ldr_color_from_json_value, ImageCache, ImageLoader};

pub mod csg;
pub mod cube;
pub mod plane;
pub mod quadratic;
pub mod quadric;
pub mod quartic;
pub mod sphere;
pub mod util;

pub fn from_json_value(
    json: &Value,
    image_cache: &ImageCache<impl ImageLoader>,
) -> Result<Box<dyn RTObject + Send + Sync>, String> {
    let dict = match json {
        Value::Object(dict) => dict,
        _ => return Err("Object must be a JSON object".to_string()),
    };

    let type_str = match dict.get("type") {
        Some(Value::String(s)) => s,
        _ => return Err("Object must have a 'type' field".to_string()),
    };

    match type_str.as_str() {
        "union" | "intersection" | "difference" => {
            csg::from_json_value(dict, type_str, image_cache)
        }
        "sphere" => sphere::from_json_value(dict, image_cache),
        "cube" => cube::from_json_value(dict, image_cache),
        "plane" => plane::from_json_value(dict, image_cache),
        _ => return Err(format!("Unknown object type: {}", type_str)),
    }
}

pub fn material_from_json_value(
    json: Option<&Value>,
    _image_cache: &ImageCache<impl ImageLoader>,
) -> Result<(LDRColor, f64, f64), String> {
    let Some(json) = json else {
        return Ok((LDRColor::new(1.0, 1.0, 1.0), 0.0, 0.0));
    };
    let Value::Object(dict) = json else {
        return Err("Material must be a JSON object".to_string());
    };
    let albedo = dict
        .get("albedo")
        .map(ldr_color_from_json_value)
        .unwrap_or(Ok(LDRColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }))?;
    let Value::Number(roughness) = dict.get("roughness").or(Some(&Value::Number(0.0))).unwrap()
    else {
        return Err("Roughness must be a number".to_string());
    };
    if *roughness < 0.0 || *roughness > 1.0 {
        return Err("Roughness must be between 0 and 1".to_string());
    }
    let Value::Number(metallic) = dict.get("metallic").or(Some(&Value::Number(0.0))).unwrap()
    else {
        return Err("Metallic must be a number".to_string());
    };
    if *metallic < 0.0 || *metallic > 1.0 {
        return Err("Metallic must be between 0 and 1".to_string());
    }
    Ok((albedo, *roughness, *metallic))
}
