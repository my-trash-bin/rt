use core::types::{
    math::{Direction, Position},
    rt::Light,
};
use types::HDRColor;

#[derive(Clone, Debug)]
pub struct DirectionalLight {
    color: HDRColor,
    direction: Direction,
}

impl Light for DirectionalLight {
    fn test(&self, _position: Position) -> Option<(HDRColor, Direction, f64)> {
        Some((self.color, -self.direction, f64::INFINITY))
    }
}

impl DirectionalLight {
    pub fn new(color: HDRColor, direction: Direction) -> Self {
        DirectionalLight { color, direction }
    }
}
