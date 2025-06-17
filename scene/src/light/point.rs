use core::types::{
    math::{Direction, Position},
    rt::Light,
};
use types::HDRColor;

#[derive(Clone, Debug)]
pub struct PointLight {
    position: Position,
    color: HDRColor,
    range: f64,
    attenuation: bool,
}

impl Light for PointLight {
    fn test(&self, position: Position) -> Option<(HDRColor, Direction, f64)> {
        // Compute the vector from the ray's origin to the light's position
        let to_light = self.position - position;
        let (direction, distance) = to_light.direction_and_length();
        if distance < 1e-3 {
            return Some((self.color, direction, distance));
        }

        // Compute attenuation using inverse square falloff
        let attenuation_factor = 1.0 / (distance * distance);
        let attenuated_color = self.color * attenuation_factor;

        Some((attenuated_color, direction, distance))
    }
}

impl PointLight {
    pub fn new(color: HDRColor, position: Position, range: f64, attenuation: bool) -> Self {
        PointLight {
            color,
            position,
            range,
            attenuation,
        }
    }
}
