use std::sync::Arc;

use types::{HDRColor, LDRColor};

use super::math::{Direction, Position};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Position,
    pub direction: Direction,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub is_front_face: bool,
    pub albedo: LDRColor,
    pub normal: Direction,
    pub distance: f64,
    pub roughness: f64,
    pub metallic: f64,
}

pub trait RTObject {
    fn test(&self, ray: Ray) -> Vec<Hit>;
}

pub trait Light {
    fn test(&self, position: Position) -> Option<(HDRColor, Direction, f64)>;
}

pub trait Camera {
    fn ray(&self, x: f64, y: f64) -> Ray;
}

pub struct Scene {
    pub image_width: usize,
    pub image_height: usize,
    pub camera: Box<dyn Camera + Send + Sync>,
    pub objects: Vec<Box<dyn RTObject + Send + Sync>>,
    pub lights: Vec<Box<dyn Light + Send + Sync>>,
    pub sky_color: Arc<dyn Fn(Direction) -> HDRColor + Send + Sync>,
    pub ambient_light: HDRColor,
}

impl Scene {
    pub fn test(&self, ray: Ray) -> Option<Hit> {
        let mut result = None::<Hit>;
        for object in self.objects.iter() {
            result = match (result, object.test(ray)) {
                (None, current) => current.first().cloned(),
                (previous, vec) if vec.is_empty() => previous,
                (Some(previous), current) => {
                    if previous.distance < current.first().unwrap().distance {
                        Some(previous)
                    } else {
                        Some(current.first().unwrap().clone())
                    }
                }
            }
        }
        result
    }
}
