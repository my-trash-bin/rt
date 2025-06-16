use super::RTObject;
use core::types::{
    math::{Direction, Position},
    rt::{Hit, Ray},
};
use types::LDRColor;

#[derive(Clone, Debug)]
pub struct Plane {
    position: Position,
    normal: Direction,
    albedo: LDRColor,
    roughness: f64,
    metallic: f64,
}

impl RTObject for Plane {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let mut result = Vec::new();

        // Compute intersection distance t
        let denominator = self.normal.dot(ray.direction);
        if denominator.abs() < 1e-6 {
            return result; // Ray is parallel to the plane, no intersection
        }

        let t = -(ray.origin.dot(*self.normal) - self.position.dot(*self.normal)) / denominator;

        if t < 0.0 {
            // The intersection is behind the ray's origin
            if self.normal.dot(ray.direction) < 0.0 {
                result.push(Hit {
                    distance: 0.0,
                    normal: -ray.direction,
                    albedo: self.albedo,
                    is_front_face: true,
                    roughness: self.roughness,
                    metallic: self.metallic,
                });
                result.push(Hit {
                    distance: f64::INFINITY,
                    normal: ray.direction,
                    albedo: self.albedo,
                    is_front_face: false,
                    roughness: self.roughness,
                    metallic: self.metallic,
                });
            }
            return result;
        }

        if self.normal.dot(ray.direction) < 0.0 {
            result.push(Hit {
                distance: t,
                normal: self.normal,
                albedo: self.albedo,
                is_front_face: true,
                roughness: self.roughness,
                metallic: self.metallic,
            });
            result.push(Hit {
                distance: f64::INFINITY,
                normal: ray.direction,
                albedo: self.albedo,
                is_front_face: false,
                roughness: self.roughness,
                metallic: self.metallic,
            });
        } else {
            result.push(Hit {
                distance: 0.0,
                normal: -ray.direction,
                albedo: self.albedo,
                is_front_face: true,
                roughness: self.roughness,
                metallic: self.metallic,
            });
            result.push(Hit {
                distance: t,
                normal: self.normal,
                albedo: self.albedo,
                is_front_face: false,
                roughness: self.roughness,
                metallic: self.metallic,
            });
        }

        result
    }
}
