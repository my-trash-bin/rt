use core::types::{
    math::{Direction, Position, Vec3},
    rt::Light,
};
use types::HDRColor;

fn down() -> Direction {
    Direction::new(-Vec3::Z)
}

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
