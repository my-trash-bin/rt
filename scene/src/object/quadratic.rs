use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Hit, Ray},
};
use types::LDRColor;

use super::{util::enhance_normal, RTObject};

#[derive(Clone, Debug)]
pub struct Quadratic {
    position: Position,
    albedo: LDRColor,
    roughness: f64,
    metallic: f64,

    c300: f64,
    c030: f64,
    c003: f64,
    c210: f64,
    c201: f64,
    c120: f64,
    c021: f64,
    c102: f64,
    c012: f64,
    c111: f64,
    c200: f64,
    c020: f64,
    c002: f64,
    c110: f64,
    c011: f64,
    c101: f64,
    c100: f64,
    c010: f64,
    c001: f64,
    c000: f64,

    inside: Position,
}

fn cubic_roots(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    if a.abs() <= 0.000001 {
        let mut roots = Vec::new();
        if b.abs() <= 0.000001 {
            if c.abs() <= 0.000001 {
                return roots;
            } else {
                roots.push(-d / c);
            }
        } else {
            let discriminant = c.powi(2) - 4.0 * b * d;
            if discriminant >= 0.0 {
                roots.push((-c + discriminant.sqrt()) / (2.0 * b));
                roots.push((-c - discriminant.sqrt()) / (2.0 * b));
            }
        }
        roots.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return roots;
    }

    let a1 = b / a;
    let a2 = c / a;
    let a3 = d / a;

    let q = (3.0 * a2 - a1 * a1) / 9.0;
    let r = (9.0 * a1 * a2 - 27.0 * a3 - 2.0 * a1 * a1 * a1) / 54.0;
    let discriminant = q * q * q + r * r;
    let a_div_3 = a1 / 3.0;

    let mut roots = Vec::new();

    if discriminant > 0.0 {
        let s = (r + discriminant.sqrt()).cbrt();
        let t = (r - discriminant.sqrt()).cbrt();
        roots.push(s + t - a_div_3);
    } else {
        let theta = (r / (-q.powi(3)).sqrt()).acos();
        if theta.is_nan() {
            return roots;
        }
        let sqrt_q = (-q).sqrt();
        roots.push(2.0 * sqrt_q * (theta / 3.0).cos() - a_div_3);
        roots.push(2.0 * sqrt_q * ((theta + 2.0 * std::f64::consts::PI) / 3.0).cos() - a_div_3);
        roots.push(2.0 * sqrt_q * ((theta - 2.0 * std::f64::consts::PI) / 3.0).cos() - a_div_3);
    }

    roots.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}

impl Quadratic {
    fn internal_test(&self, ray: Ray) -> Vec<Hit> {
        let origin: Position = (ray.origin - self.position).into();

        let (a, b, c, d) = {
            let mut a = 0.0;
            let mut b = 0.0;
            let mut c = 0.0;
            let mut d = 0.0;
            // c300
            a += self.c300 * ray.direction.x.powi(3);
            b += self.c300 * 3.0 * ray.direction.x.powi(2) * origin.x;
            c += self.c300 * 3.0 * ray.direction.x * origin.x.powi(2);
            d += self.c300 * origin.x.powi(3);
            // c030
            a += self.c030 * ray.direction.y.powi(3);
            b += self.c030 * 3.0 * ray.direction.y.powi(2) * origin.y;
            c += self.c030 * 3.0 * ray.direction.y * origin.y.powi(2);
            d += self.c030 * origin.y.powi(3);
            // c003
            a += self.c003 * ray.direction.z.powi(3);
            b += self.c003 * 3.0 * ray.direction.z.powi(2) * origin.z;
            c += self.c003 * 3.0 * ray.direction.z * origin.z.powi(2);
            d += self.c003 * origin.z.powi(3);
            // c210
            a += self.c210 * ray.direction.x.powi(2) * ray.direction.y;
            b += self.c210 * ray.direction.x.powi(2) * origin.y;
            b += self.c210 * 2.0 * ray.direction.x * ray.direction.y * origin.x;
            c += self.c210 * 2.0 * ray.direction.x * origin.x * origin.y;
            c += self.c210 * ray.direction.y * origin.x.powi(2);
            d += self.c210 * origin.x.powi(2) * origin.y;
            // c201
            a += self.c201 * ray.direction.x.powi(2) * ray.direction.z;
            b += self.c201 * ray.direction.x.powi(2) * origin.z;
            b += self.c201 * 2.0 * ray.direction.x * ray.direction.z * origin.x;
            c += self.c201 * 2.0 * ray.direction.x * origin.x * origin.z;
            c += self.c201 * ray.direction.z * origin.x.powi(2);
            d += self.c201 * origin.x.powi(2) * origin.z;
            // c120
            a += self.c120 * ray.direction.y.powi(2) * ray.direction.x;
            b += self.c120 * ray.direction.y.powi(2) * origin.x;
            b += self.c120 * 2.0 * ray.direction.y * ray.direction.x * origin.y;
            c += self.c120 * 2.0 * ray.direction.y * origin.y * origin.x;
            c += self.c120 * ray.direction.x * origin.y.powi(2);
            d += self.c120 * origin.y.powi(2) * origin.x;
            // c021
            a += self.c021 * ray.direction.y.powi(2) * ray.direction.z;
            b += self.c021 * ray.direction.y.powi(2) * origin.z;
            b += self.c021 * 2.0 * ray.direction.y * ray.direction.z * origin.y;
            c += self.c021 * 2.0 * ray.direction.y * origin.y * origin.z;
            c += self.c021 * ray.direction.z * origin.y.powi(2);
            d += self.c021 * origin.y.powi(2) * origin.z;
            // c102
            a += self.c102 * ray.direction.z.powi(2) * ray.direction.x;
            b += self.c102 * ray.direction.z.powi(2) * origin.x;
            b += self.c102 * 2.0 * ray.direction.z * ray.direction.x * origin.z;
            c += self.c102 * 2.0 * ray.direction.z * origin.z * origin.x;
            c += self.c102 * ray.direction.x * origin.z.powi(2);
            d += self.c102 * origin.z.powi(2) * origin.x;
            // c012
            a += self.c012 * ray.direction.z.powi(2) * ray.direction.y;
            b += self.c012 * ray.direction.z.powi(2) * origin.y;
            b += self.c012 * 2.0 * ray.direction.z * ray.direction.y * origin.z;
            c += self.c012 * 2.0 * ray.direction.z * origin.z * origin.y;
            c += self.c012 * ray.direction.y * origin.z.powi(2);
            d += self.c012 * origin.z.powi(2) * origin.y;
            // c111
            a += self.c111 * ray.direction.x * ray.direction.y * ray.direction.z;
            b += self.c111 * ray.direction.x * ray.direction.y * origin.z;
            b += self.c111 * ray.direction.x * origin.y * ray.direction.z;
            b += self.c111 * origin.x * ray.direction.y * ray.direction.z;
            c += self.c111 * ray.direction.x * origin.y * origin.z;
            c += self.c111 * origin.x * ray.direction.y * origin.z;
            c += self.c111 * origin.x * origin.y * ray.direction.z;
            d += self.c111 * origin.x * origin.y * origin.z;
            // c200
            b += self.c200 * ray.direction.x.powi(2);
            c += self.c200 * 2.0 * ray.direction.x * origin.x;
            d += self.c200 * origin.x.powi(2);
            // c020
            b += self.c020 * ray.direction.y.powi(2);
            c += self.c020 * 2.0 * ray.direction.y * origin.y;
            d += self.c020 * origin.y.powi(2);
            // c002
            b += self.c002 * ray.direction.z.powi(2);
            c += self.c002 * 2.0 * ray.direction.z * origin.z;
            d += self.c002 * origin.z.powi(2);
            // c110
            b += self.c110 * ray.direction.x * ray.direction.y;
            c += self.c110 * ray.direction.x * origin.y;
            c += self.c110 * origin.x * ray.direction.y;
            d += self.c110 * origin.x * origin.y;
            // c011
            b += self.c011 * ray.direction.y * ray.direction.z;
            c += self.c011 * ray.direction.y * origin.z;
            c += self.c011 * origin.y * ray.direction.z;
            d += self.c011 * origin.y * origin.z;
            // c101
            b += self.c101 * ray.direction.x * ray.direction.z;
            c += self.c101 * ray.direction.x * origin.z;
            c += self.c101 * origin.x * ray.direction.z;
            d += self.c101 * origin.x * origin.z;
            // c100
            c += self.c100 * ray.direction.x;
            d += self.c100 * origin.x;
            // c010
            c += self.c010 * ray.direction.y;
            d += self.c010 * origin.y;
            // c001
            c += self.c001 * ray.direction.z;
            d += self.c001 * origin.z;
            // c000
            d += self.c000;
            // done
            (a, b, c, d)
        };

        cubic_roots(a, b, c, d)
            .into_iter()
            .filter(|t| *t >= 0.0)
            .map(|distance| Hit {
                distance,
                normal: self.normal(origin + ray.direction * distance),
                albedo: self.albedo,
                is_front_face: true, // decided later
                roughness: self.roughness,
                metallic: self.metallic,
            })
            .collect()
    }

    fn normal(&self, position: Position) -> Direction {
        Direction::new(Vec3::new(
            3.0 * self.c300 * position.x.powi(2)
                + 2.0 * self.c210 * position.x * position.y
                + 2.0 * self.c201 * position.x * position.z
                + 2.0 * self.c200 * position.x
                + self.c120 * position.y.powi(2)
                + self.c102 * position.z.powi(2)
                + self.c111 * position.y * position.z
                + self.c110 * position.y
                + self.c101 * position.z
                + self.c100,
            3.0 * self.c030 * position.y.powi(2)
                + 2.0 * self.c120 * position.y * position.x
                + 2.0 * self.c021 * position.y * position.z
                + 2.0 * self.c020 * position.y
                + self.c210 * position.x.powi(2)
                + self.c012 * position.z.powi(2)
                + self.c111 * position.x * position.z
                + self.c110 * position.x
                + self.c011 * position.z
                + self.c010,
            3.0 * self.c003 * position.z.powi(2)
                + 2.0 * self.c102 * position.z * position.x
                + 2.0 * self.c012 * position.z * position.y
                + 2.0 * self.c002 * position.z
                + self.c201 * position.x.powi(2)
                + self.c021 * position.y.powi(2)
                + self.c111 * position.x * position.y
                + self.c101 * position.x
                + self.c011 * position.y
                + self.c001,
        ))
    }
}

impl RTObject for Quadratic {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let (inside_direction, inside_length) = (ray.origin - self.inside).direction_and_length();
        let internal = self.internal_test(Ray {
            origin: self.inside,
            direction: inside_direction,
        });
        let inside = internal
            .into_iter()
            .filter(|hit| hit.distance < inside_length)
            .count()
            % 2
            == 0;

        let mut result = Vec::new();

        let mut is_front_face = false;
        if inside {
            is_front_face = true;
            result.push(Hit {
                distance: 0.0,
                normal: -ray.direction,
                albedo: self.albedo,
                is_front_face,
                roughness: self.roughness,
                metallic: self.metallic,
            });
        }
        for hit in self.internal_test(ray).into_iter() {
            is_front_face = !is_front_face;
            result.push(Hit {
                normal: enhance_normal(ray.direction, hit.normal, is_front_face),
                is_front_face,
                ..hit
            });
        }
        if is_front_face {
            result.push(Hit {
                distance: f64::INFINITY,
                normal: ray.direction,
                albedo: self.albedo,
                is_front_face: false,
                roughness: self.roughness,
                metallic: self.metallic,
            });
        }

        result
    }
}
