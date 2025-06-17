use core::types::{
    math::{Direction, Position},
    rt::Light,
};
use types::HDRColor;

#[derive(Clone, Debug)]
pub struct SpotLight {
    color: HDRColor,
    position: Position,
    angle: f64,
    direction: Direction,
    range: f64,
    attenuation: bool,
}

impl SpotLight {
    pub fn new(
        color: HDRColor,
        position: Position,
        angle: f64,
        direction: Direction,
        range: f64,
        attenuation: bool,
    ) -> Self {
        SpotLight {
            color,
            position,
            angle,
            direction,
            range,
            attenuation,
        }
    }
}

impl Light for SpotLight {
    fn test(&self, _position: Position) -> Option<(HDRColor, Direction, f64)> {
        // TODO: Spot light 구현
        None
    }
}
