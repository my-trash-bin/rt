use super::RTObject;

use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Hit, Ray},
};
use types::LDRColor;

#[derive(Clone, Debug)]
pub struct Cube {
    position: Position,
    albedo: LDRColor,
    scale: Vec3,
    roughness: f64,
    metallic: f64,
}

impl RTObject for Cube {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let mut result = Vec::new();

        let min = Position::new(Vec3::new(
            self.position.x - self.scale.x,
            self.position.y - self.scale.y,
            self.position.z - self.scale.z,
        ));
        let max = Position::new(Vec3::new(
            self.position.x + self.scale.x,
            self.position.y + self.scale.y,
            self.position.z + self.scale.z,
        ));

        let mut t_min = f64::NEG_INFINITY;
        let mut t_max = f64::INFINITY;
        let mut normal_min = Vec3::ZERO;
        let mut normal_max = Vec3::ZERO;

        for (i, (o, d, min, max)) in [
            (ray.origin.x, ray.direction.x, min.x, max.x),
            (ray.origin.y, ray.direction.y, min.y, max.y),
            (ray.origin.z, ray.direction.z, min.z, max.z),
        ]
        .iter()
        .enumerate()
        {
            if *d != 0.0 {
                let t1 = (min - o) / d;
                let t2 = (max - o) / d;
                let (t1, t2, normal1, normal2) = if t1 < t2 {
                    (
                        t1,
                        t2,
                        -Vec3::X * (i == 0) as i32 as f64
                            - Vec3::Y * (i == 1) as i32 as f64
                            - Vec3::Z * (i == 2) as i32 as f64,
                        Vec3::X * (i == 0) as i32 as f64
                            + Vec3::Y * (i == 1) as i32 as f64
                            + Vec3::Z * (i == 2) as i32 as f64,
                    )
                } else {
                    (
                        t2,
                        t1,
                        Vec3::X * (i == 0) as i32 as f64
                            + Vec3::Y * (i == 1) as i32 as f64
                            + Vec3::Z * (i == 2) as i32 as f64,
                        -Vec3::X * (i == 0) as i32 as f64
                            - Vec3::Y * (i == 1) as i32 as f64
                            - Vec3::Z * (i == 2) as i32 as f64,
                    )
                };

                if t1 > t_min {
                    t_min = t1;
                    normal_min = normal1;
                }
                if t2 < t_max {
                    t_max = t2;
                    normal_max = normal2;
                }
                if t_min > t_max {
                    return result;
                }
            } else if *o < *min || *o > *max {
                return result;
            }
        }

        if t_min < 0.0 && t_max < 0.0 {
            return result;
        }

        if t_min < 0.0 {
            t_min = 0.0;
        }

        if t_min <= t_max {
            if t_min >= 0.0 {
                result.push(Hit {
                    normal: Direction::new(normal_min),
                    distance: t_min,
                    is_front_face: true,
                    albedo: self.albedo,
                    roughness: self.roughness,
                    metallic: self.metallic,
                });
            }
            if t_max >= 0.0 {
                result.push(Hit {
                    normal: Direction::new(normal_max),
                    distance: t_max,
                    is_front_face: false,
                    albedo: self.albedo,
                    roughness: self.roughness,
                    metallic: self.metallic,
                });
            }
        }

        result
    }
}
