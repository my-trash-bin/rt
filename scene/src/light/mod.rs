use core::types::math::{Direction, Position, Vec3};
use core::types::rt::Light;
use directional::DirectionalLight;
use jsonc::Value;
use point::PointLight;
use spot::SpotLight;
use types::HDRColor;
pub mod directional;
pub mod point;
pub mod spot;

pub enum DeserializableLight {
    Point(PointLight),
    Directional(DirectionalLight),
    Spot(SpotLight),
}

impl DeserializableLight {
    pub fn into_light(self) -> Box<dyn Light + Send + Sync> {
        match self {
            DeserializableLight::Point(c) => Box::new(c),
            DeserializableLight::Directional(c) => Box::new(c),
            DeserializableLight::Spot(c) => Box::new(c),
        }
    }

    pub fn from_json(json: &Value) -> Result<DeserializableLight, String> {
        let dict = match json {
            Value::Object(dict) => dict,
            _ => return Err("Light must be a JSON object".to_string()),
        };

        let type_str = match dict.get("type") {
            Some(Value::String(s)) => s,
            _ => return Err("Light must have a 'type' field with string value".to_string()),
        };

        match type_str.as_str() {
            "point" => {
                let color_json = dict.get("color").ok_or("Missing required field: color")?;
                let color = Self::parse_hdr_color(color_json)?;

                let position_json = dict
                    .get("position")
                    .ok_or("Missing required field: position")?;
                let position = Self::parse_position(position_json)?;

                let range = if let Some(Value::Number(r)) = dict.get("range") {
                    if *r <= 0.0 {
                        return Err("range must be greater than 0".to_string());
                    }
                    *r
                } else {
                    f64::INFINITY
                };

                let attenuation = if let Some(Value::Bool(a)) = dict.get("attenuation") {
                    *a
                } else {
                    true
                };

                Ok(DeserializableLight::Point(PointLight::new(
                    color,
                    position,
                    range,
                    attenuation,
                )))
            }
            "directional" => {
                let color_json = dict.get("color").ok_or("Missing required field: color")?;
                let color = Self::parse_hdr_color(color_json)?;

                let direction_json = dict
                    .get("direction")
                    .ok_or("Missing required field: direction")?;
                let direction = Self::parse_direction(direction_json)?;

                Ok(DeserializableLight::Directional(DirectionalLight::new(
                    color, direction,
                )))
            }
            "spot" => {
                let color_json = dict.get("color").ok_or("Missing required field: color")?;
                let color = Self::parse_hdr_color(color_json)?;

                let position_json = dict
                    .get("position")
                    .ok_or("Missing required field: position")?;
                let position = Self::parse_position(position_json)?;

                let angle_json = dict.get("angle").ok_or("Missing required field: angle")?;
                let angle = Self::parse_angle(angle_json)?;

                let direction_json = dict
                    .get("direction")
                    .ok_or("Missing required field: direction")?;
                let direction = Self::parse_direction(direction_json)?;

                let range = if let Some(Value::Number(r)) = dict.get("range") {
                    if *r <= 0.0 {
                        return Err("range must be greater than 0".to_string());
                    }
                    *r
                } else {
                    f64::INFINITY
                };

                let attenuation = if let Some(Value::Bool(a)) = dict.get("attenuation") {
                    *a
                } else {
                    true
                };

                Ok(DeserializableLight::Spot(SpotLight::new(
                    color,
                    position,
                    angle,
                    direction,
                    range,
                    attenuation,
                )))
            }
            _ => Err(format!("Unknown light type: {}", type_str)),
        }
    }

    fn parse_hdr_color(json: &Value) -> Result<HDRColor, String> {
        match json {
            Value::Array(array) if array.len() == 3 => {
                let r = match &array[0] {
                    Value::Number(n) => {
                        if *n < 0.0 || !n.is_finite() {
                            return Err("color[0] must be a non-negative finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("color[0] must be a number".to_string()),
                };
                let g = match &array[1] {
                    Value::Number(n) => {
                        if *n < 0.0 || !n.is_finite() {
                            return Err("color[1] must be a non-negative finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("color[1] must be a number".to_string()),
                };
                let b = match &array[2] {
                    Value::Number(n) => {
                        if *n < 0.0 || !n.is_finite() {
                            return Err("color[2] must be a non-negative finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("color[2] must be a number".to_string()),
                };
                Ok(HDRColor { r, g, b })
            }
            _ => Err("color must be an array of 3 numbers".to_string()),
        }
    }

    fn parse_position(json: &Value) -> Result<Position, String> {
        match json {
            Value::Array(array) if array.len() == 3 => {
                let x = match &array[0] {
                    Value::Number(n) => *n,
                    _ => return Err("position[0] must be a number".to_string()),
                };
                let y = match &array[1] {
                    Value::Number(n) => *n,
                    _ => return Err("position[1] must be a number".to_string()),
                };
                let z = match &array[2] {
                    Value::Number(n) => *n,
                    _ => return Err("position[2] must be a number".to_string()),
                };
                Ok(Position::new(Vec3::new(x, y, z)))
            }
            _ => Err("position must be an array of 3 numbers".to_string()),
        }
    }

    fn parse_direction(json: &Value) -> Result<Direction, String> {
        match json {
            Value::Array(array) if array.len() == 3 => {
                let x = match &array[0] {
                    Value::Number(n) => {
                        if !n.is_finite() {
                            return Err("direction[0] must be a finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("direction[0] must be a number".to_string()),
                };
                let y = match &array[1] {
                    Value::Number(n) => {
                        if !n.is_finite() {
                            return Err("direction[1] must be a finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("direction[1] must be a number".to_string()),
                };
                let z = match &array[2] {
                    Value::Number(n) => {
                        if !n.is_finite() {
                            return Err("direction[2] must be a finite number".to_string());
                        }
                        *n
                    }
                    _ => return Err("direction[2] must be a number".to_string()),
                };

                let vec = Vec3::new(x, y, z);
                if vec.length() < 1e-10 {
                    return Err("direction vector cannot be zero or near-zero".to_string());
                }

                Ok(Direction::new(vec))
            }
            _ => Err("direction must be an array of 3 numbers".to_string()),
        }
    }

    fn parse_angle(json: &Value) -> Result<f64, String> {
        let dict = match json {
            Value::Object(dict) => dict,
            _ => return Err("angle must be a JSON object".to_string()),
        };

        if let Some(Value::Number(degree)) = dict.get("degree") {
            Ok(*degree)
        } else if let Some(Value::Number(radian)) = dict.get("radian") {
            Ok(radian.to_degrees())
        } else {
            Err("angle must have either 'degree' or 'radian' field".to_string())
        }
    }
}
