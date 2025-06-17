use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Camera, Ray},
};
use json::JsonValue;

#[derive(Clone, Debug)]
pub struct AspectRatio {
    aspect_ratio: f64,
}

#[derive(Clone, Debug)]
pub enum FovMode {
    X,
    Y,
    Cover(AspectRatio),
    Contain(AspectRatio),
}

#[derive(Clone, Debug)]
pub struct DeserializablePerspectiveCamera {
    fov: f64,
    fov_mode: FovMode,
    position: Position,
    direction: Direction,
}

impl DeserializablePerspectiveCamera {
    pub fn from_json(json: &JsonValue) -> Result<DeserializablePerspectiveCamera, String> {
        let dict = match json {
            JsonValue::Dict(dict) => dict,
            _ => return Err("Camera must be a JSON object".to_string()),
        };

        let fov_json = dict.get("fov").ok_or("Missing required field: fov")?;
        let (fov, fov_mode) = Self::parse_fov(fov_json)?;

        let position_json = dict
            .get("position")
            .ok_or("Missing required field: position")?;
        let position = Self::parse_position(position_json)?;

        let direction = if let Some(direction_json) = dict.get("direction") {
            Self::parse_direction(direction_json)?
        } else if let Some(look_at_json) = dict.get("lookAt") {
            let look_at = Self::parse_position(look_at_json)?;
            Direction::new(*(look_at - position))
        } else {
            return Err("Camera must have either 'direction' or 'lookAt' field".to_string());
        };

        Ok(DeserializablePerspectiveCamera {
            fov,
            fov_mode,
            position,
            direction,
        })
    }

    fn parse_fov(json: &JsonValue) -> Result<(f64, FovMode), String> {
        let dict = match json {
            JsonValue::Dict(dict) => dict,
            _ => return Err("fov must be a JSON object".to_string()),
        };

        if let Some(x_json) = dict.get("x") {
            let angle = Self::parse_angle(x_json)?;
            Ok((angle, FovMode::X))
        } else if let Some(y_json) = dict.get("y") {
            let angle = Self::parse_angle(y_json)?;
            Ok((angle, FovMode::Y))
        } else if let Some(min_json) = dict.get("min") {
            let angle = Self::parse_angle(min_json)?;
            Ok((angle, FovMode::Contain(AspectRatio { aspect_ratio: 1.0 })))
        } else if let Some(max_json) = dict.get("max") {
            let angle = Self::parse_angle(max_json)?;
            Ok((angle, FovMode::Cover(AspectRatio { aspect_ratio: 1.0 })))
        } else {
            Err("fov must have one of: 'x', 'y', 'min', or 'max' field".to_string())
        }
    }

    fn parse_angle(json: &JsonValue) -> Result<f64, String> {
        let dict = match json {
            JsonValue::Dict(dict) => dict,
            _ => return Err("angle must be a JSON object".to_string()),
        };

        if let Some(JsonValue::Number(degree)) = dict.get("degree") {
            Ok(*degree)
        } else if let Some(JsonValue::Number(radian)) = dict.get("radian") {
            Ok(radian.to_degrees())
        } else {
            Err("angle must have either 'degree' or 'radian' field".to_string())
        }
    }

    fn parse_position(json: &JsonValue) -> Result<Position, String> {
        let array = match json {
            JsonValue::List(array) if array.len() == 3 => array,
            _ => return Err("position must be an array of 3 numbers".to_string()),
        };

        let x = match &array[0] {
            JsonValue::Number(n) => *n,
            _ => return Err("position[0] must be a number".to_string()),
        };
        let y = match &array[1] {
            JsonValue::Number(n) => *n,
            _ => return Err("position[1] must be a number".to_string()),
        };
        let z = match &array[2] {
            JsonValue::Number(n) => *n,
            _ => return Err("position[2] must be a number".to_string()),
        };

        Ok(Position::new(Vec3::new(x, y, z)))
    }

    fn parse_direction(json: &JsonValue) -> Result<Direction, String> {
        let array = match json {
            JsonValue::List(array) if array.len() == 3 => array,
            _ => return Err("direction must be an array of 3 numbers".to_string()),
        };

        let x = match &array[0] {
            JsonValue::Number(n) => *n,
            _ => return Err("direction[0] must be a number".to_string()),
        };
        let y = match &array[1] {
            JsonValue::Number(n) => *n,
            _ => return Err("direction[1] must be a number".to_string()),
        };
        let z = match &array[2] {
            JsonValue::Number(n) => *n,
            _ => return Err("direction[2] must be a number".to_string()),
        };

        Ok(Direction::new(Vec3::new(x, y, z)))
    }

    pub fn into_camera(self, screen_aspect_ratio: f64) -> Box<dyn Camera + Send + Sync> {
        let (tan_half_fov_x, tan_half_fov_y) = match self.fov_mode {
            FovMode::X => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / screen_aspect_ratio;
                (tan_half_fov_x, tan_half_fov_y)
            }
            FovMode::Y => {
                let tan_half_fov_y = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_x = tan_half_fov_y * screen_aspect_ratio;
                (tan_half_fov_x, tan_half_fov_y)
            }
            FovMode::Cover(aspect_ratio) => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / aspect_ratio.aspect_ratio;
                let scale = (screen_aspect_ratio / aspect_ratio.aspect_ratio).max(1.0);
                (tan_half_fov_x * scale, tan_half_fov_y * scale)
            }
            FovMode::Contain(aspect_ratio) => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / aspect_ratio.aspect_ratio;
                let scale = (screen_aspect_ratio / aspect_ratio.aspect_ratio).min(1.0);
                (tan_half_fov_x * scale, tan_half_fov_y * scale)
            }
        };

        let right = self.direction.cross(Vec3::Z).normalize();
        let up = right.cross(*self.direction).normalize();
        Box::new(PerspectiveCamera {
            tan_half_fov_x,
            tan_half_fov_y,
            position: self.position,
            direction: self.direction,
            right,
            up,
        })
    }
}

struct PerspectiveCamera {
    tan_half_fov_x: f64,
    tan_half_fov_y: f64,
    position: Position,
    direction: Direction,
    right: Vec3,
    up: Vec3,
}

impl Camera for PerspectiveCamera {
    fn ray(&self, x: f64, y: f64) -> Ray {
        let dir_x = (2.0 * x - 1.0) * self.tan_half_fov_x;
        let dir_z = (1.0 - 2.0 * y) * self.tan_half_fov_y;

        let direction = Direction::new(*self.direction + dir_x * self.right + self.up * dir_z);

        Ray {
            origin: self.position,
            direction,
        }
    }
}
