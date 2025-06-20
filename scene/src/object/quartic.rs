use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Hit, Ray},
};
use types::LDRColor;

use super::{util::enhance_normal, RTObject};

#[derive(Clone, Debug)]
pub struct Quartic {
    pub position: Position,
    pub albedo: LDRColor,
    pub roughness: f64,
    pub metallic: f64,

    pub c400: f64,
    pub c040: f64,
    pub c004: f64,
    pub c310: f64,
    pub c301: f64,
    pub c130: f64,
    pub c031: f64,
    pub c103: f64,
    pub c013: f64,
    pub c211: f64,
    pub c121: f64,
    pub c112: f64,
    pub c220: f64,
    pub c022: f64,
    pub c202: f64,
    pub c300: f64,
    pub c030: f64,
    pub c003: f64,
    pub c210: f64,
    pub c201: f64,
    pub c120: f64,
    pub c021: f64,
    pub c102: f64,
    pub c012: f64,
    pub c111: f64,
    pub c200: f64,
    pub c020: f64,
    pub c002: f64,
    pub c110: f64,
    pub c011: f64,
    pub c101: f64,
    pub c100: f64,
    pub c010: f64,
    pub c001: f64,
    pub c000: f64,

    pub point: Position,
    pub is_point_inside: bool,
}

fn linear_roots(a: f64, b: f64) -> Vec<f64> {
    if b.abs() <= 1e-6 {
        vec![]
    } else {
        vec![-b / a]
    }
}

fn quadratic_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a.abs() <= 1e-6 {
        return linear_roots(b, c);
    }

    let mut roots = Vec::new();

    let discriminant = b * b - 4.0 * a * c;
    if discriminant >= 0.0 {
        roots.push((-b + discriminant.sqrt()) / (2.0 * a));
        roots.push((-b - discriminant.sqrt()) / (2.0 * a));
    }

    roots.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}

fn cubic_roots(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    if a.abs() <= 1e-6 {
        return quadratic_roots(b, c, d);
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

fn quartic_roots(a: f64, b: f64, c: f64, d: f64, e: f64) -> Vec<f64> {
    if a.abs() <= 1e-6 {
        return cubic_roots(b, c, d, e);
    }

    let b = b / a;
    let c = c / a;
    let d = d / a;
    let e = e / a;

    let bb = b * b;
    let p = -3.0 * bb / 8.0 + c;
    let q = bb * b / 8.0 - b * c / 2.0 + d;
    let r = -3.0 * bb * bb / 256.0 + bb * c / 16.0 - b * d / 4.0 + e;

    let mut roots = Vec::new();

    if q.abs() < 1e-6 {
        let discriminant1 = p * p - 4.0 * r;
        if discriminant1 >= 0.0 {
            let y1 = (-p + discriminant1.sqrt()) / 2.0;
            let y2 = (-p - discriminant1.sqrt()) / 2.0;

            for y in [y1, y2].iter() {
                if *y >= 0.0 {
                    roots.push(y.sqrt() - b / 4.0);
                    roots.push(-y.sqrt() - b / 4.0);
                }
            }
        }
    } else {
        let cubic_a = 1.0;
        let cubic_b = -p / 2.0;
        let cubic_c = -r;
        let cubic_d = (p * r - q * q / 4.0) / 2.0;

        let cubic_roots = cubic_roots(cubic_a, cubic_b, cubic_c, cubic_d);
        if cubic_roots.is_empty() {
            return roots;
        }

        let z = cubic_roots[0];
        let u = (2.0 * z - p).sqrt();
        let v = if u.abs() > 1e-6 { q / (2.0 * u) } else { 0.0 };

        let quadratic1_a = 1.0;
        let quadratic1_b = u;
        let quadratic1_c = z - v;

        let quadratic2_a = 1.0;
        let quadratic2_b = -u;
        let quadratic2_c = z + v;

        for (qa, qb, qc) in [
            (quadratic1_a, quadratic1_b, quadratic1_c),
            (quadratic2_a, quadratic2_b, quadratic2_c),
        ]
        .iter()
        {
            let disc = qb * qb - 4.0 * qa * qc;
            if disc >= 0.0 {
                roots.push((-qb + disc.sqrt()) / (2.0 * qa) - b / 4.0);
                roots.push((-qb - disc.sqrt()) / (2.0 * qa) - b / 4.0);
            }
        }
    }

    roots.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}

impl Quartic {
    fn internal_test(&self, ray: Ray) -> Vec<Hit> {
        let origin: Position = (ray.origin - self.position).into();

        let (a, b, c, d, e) = {
            let mut a = 0.0;
            let mut b = 0.0;
            let mut c = 0.0;
            let mut d = 0.0;
            let mut e = 0.0;
            // c400
            a += self.c400 * ray.direction.x.powi(4);
            b += self.c400 * 4.0 * ray.direction.x.powi(3) * origin.x;
            c += self.c400 * 6.0 * ray.direction.x.powi(2) * origin.x.powi(2);
            d += self.c400 * 4.0 * ray.direction.x * origin.x.powi(3);
            e += self.c400 * origin.x.powi(4);
            // c040
            a += self.c040 * ray.direction.y.powi(4);
            b += self.c040 * 4.0 * ray.direction.y.powi(3) * origin.y;
            c += self.c040 * 6.0 * ray.direction.y.powi(2) * origin.y.powi(2);
            d += self.c040 * 4.0 * ray.direction.y * origin.y.powi(3);
            e += self.c040 * origin.y.powi(4);
            // c004
            a += self.c004 * ray.direction.z.powi(4);
            b += self.c004 * 4.0 * ray.direction.z.powi(3) * origin.z;
            c += self.c004 * 6.0 * ray.direction.z.powi(2) * origin.z.powi(2);
            d += self.c004 * 4.0 * ray.direction.z * origin.z.powi(3);
            e += self.c004 * origin.z.powi(4);
            // c310
            a += self.c310 * ray.direction.x.powi(3) * ray.direction.y;
            b += self.c310 * 3.0 * ray.direction.x.powi(3) * origin.y;
            b += self.c310 * ray.direction.x.powi(2) * ray.direction.y * origin.x;
            c += self.c310 * 3.0 * ray.direction.x.powi(2) * origin.x * origin.y;
            c += self.c310 * 3.0 * ray.direction.x * ray.direction.y * origin.x.powi(2);
            d += self.c310 * 3.0 * ray.direction.x * origin.x.powi(2) * origin.y;
            d += self.c310 * ray.direction.y * origin.x.powi(3);
            e += self.c310 * origin.x.powi(3) * origin.y;
            // c301
            a += self.c301 * ray.direction.x.powi(3) * ray.direction.z;
            b += self.c301 * 3.0 * ray.direction.x.powi(3) * origin.z;
            b += self.c301 * ray.direction.x.powi(2) * ray.direction.z * origin.x;
            c += self.c301 * 3.0 * ray.direction.x.powi(2) * origin.x * origin.z;
            c += self.c301 * 3.0 * ray.direction.x * ray.direction.z * origin.x.powi(2);
            d += self.c301 * 3.0 * ray.direction.x * origin.x.powi(2) * origin.z;
            d += self.c301 * ray.direction.z * origin.x.powi(3);
            e += self.c301 * origin.x.powi(3) * origin.z;
            // c130
            a += self.c130 * ray.direction.y.powi(3) * ray.direction.x;
            b += self.c130 * 3.0 * ray.direction.y.powi(3) * origin.x;
            b += self.c130 * ray.direction.y.powi(2) * ray.direction.x * origin.y;
            c += self.c130 * 3.0 * ray.direction.y.powi(2) * origin.y * origin.x;
            c += self.c130 * 3.0 * ray.direction.y * ray.direction.x * origin.y.powi(2);
            d += self.c130 * 3.0 * ray.direction.y * origin.y.powi(2) * origin.x;
            d += self.c130 * ray.direction.x * origin.y.powi(3);
            e += self.c130 * origin.y.powi(3) * origin.x;
            // c031
            a += self.c031 * ray.direction.y.powi(3) * ray.direction.z;
            b += self.c031 * 3.0 * ray.direction.y.powi(3) * origin.z;
            b += self.c031 * ray.direction.y.powi(2) * ray.direction.z * origin.y;
            c += self.c031 * 3.0 * ray.direction.y.powi(2) * origin.y * origin.z;
            c += self.c031 * 3.0 * ray.direction.y * ray.direction.z * origin.y.powi(2);
            d += self.c031 * 3.0 * ray.direction.y * origin.y.powi(2) * origin.z;
            d += self.c031 * ray.direction.z * origin.y.powi(3);
            e += self.c031 * origin.y.powi(3) * origin.z;
            // c103
            a += self.c103 * ray.direction.z.powi(3) * ray.direction.x;
            b += self.c103 * 3.0 * ray.direction.z.powi(3) * origin.x;
            b += self.c103 * ray.direction.z.powi(2) * ray.direction.x * origin.z;
            c += self.c103 * 3.0 * ray.direction.z.powi(2) * origin.z * origin.x;
            c += self.c103 * 3.0 * ray.direction.z * ray.direction.x * origin.z.powi(2);
            d += self.c103 * 3.0 * ray.direction.z * origin.z.powi(2) * origin.x;
            d += self.c103 * ray.direction.x * origin.z.powi(3);
            e += self.c103 * origin.z.powi(3) * origin.x;
            // c013
            a += self.c013 * ray.direction.z.powi(3) * ray.direction.y;
            b += self.c013 * 3.0 * ray.direction.z.powi(3) * origin.y;
            b += self.c013 * ray.direction.z.powi(2) * ray.direction.y * origin.z;
            c += self.c013 * 3.0 * ray.direction.z.powi(2) * origin.z * origin.y;
            c += self.c013 * 3.0 * ray.direction.z * ray.direction.y * origin.z.powi(2);
            d += self.c013 * 3.0 * ray.direction.z * origin.z.powi(2) * origin.y;
            d += self.c013 * ray.direction.y * origin.z.powi(3);
            e += self.c013 * origin.z.powi(3) * origin.y;
            // c211
            a += self.c211 * ray.direction.x.powi(2) * ray.direction.y * ray.direction.z;
            b += self.c211 * 2.0 * ray.direction.x * ray.direction.y * ray.direction.z * origin.x;
            b += self.c211 * ray.direction.x.powi(2) * ray.direction.z * origin.y;
            b += self.c211 * ray.direction.x.powi(2) * ray.direction.y * origin.z;
            c += self.c211 * ray.direction.x.powi(2) * origin.y * origin.z;
            c += self.c211 * 2.0 * ray.direction.x * ray.direction.y * origin.x * origin.z;
            c += self.c211 * 2.0 * ray.direction.x * ray.direction.z * origin.x * origin.y;
            c += self.c211 * ray.direction.y * ray.direction.z * origin.x.powi(2);
            d += self.c211 * 2.0 * ray.direction.x * origin.x * origin.y * origin.z;
            d += self.c211 * ray.direction.y * origin.x.powi(2) * origin.z;
            d += self.c211 * ray.direction.z * origin.x.powi(2) * origin.y;
            e += self.c211 * origin.x.powi(2) * origin.y * origin.z;
            // c121
            a += self.c121 * ray.direction.y.powi(2) * ray.direction.x * ray.direction.z;
            b += self.c121 * 2.0 * ray.direction.y * ray.direction.x * ray.direction.z * origin.y;
            b += self.c121 * ray.direction.y.powi(2) * ray.direction.z * origin.x;
            b += self.c121 * ray.direction.y.powi(2) * ray.direction.x * origin.z;
            c += self.c121 * ray.direction.y.powi(2) * origin.x * origin.z;
            c += self.c121 * 2.0 * ray.direction.y * ray.direction.x * origin.y * origin.z;
            c += self.c121 * 2.0 * ray.direction.y * ray.direction.z * origin.y * origin.x;
            c += self.c121 * ray.direction.x * ray.direction.z * origin.y.powi(2);
            d += self.c121 * 2.0 * ray.direction.y * origin.y * origin.x * origin.z;
            d += self.c121 * ray.direction.x * origin.y.powi(2) * origin.z;
            d += self.c121 * ray.direction.z * origin.y.powi(2) * origin.x;
            e += self.c121 * origin.y.powi(2) * origin.x * origin.z;
            // c112
            a += self.c112 * ray.direction.z.powi(2) * ray.direction.x * ray.direction.y;
            b += self.c112 * 2.0 * ray.direction.z * ray.direction.x * ray.direction.y * origin.z;
            b += self.c112 * ray.direction.z.powi(2) * ray.direction.y * origin.x;
            b += self.c112 * ray.direction.z.powi(2) * ray.direction.x * origin.y;
            c += self.c112 * ray.direction.z.powi(2) * origin.x * origin.y;
            c += self.c112 * 2.0 * ray.direction.z * ray.direction.x * origin.z * origin.y;
            c += self.c112 * 2.0 * ray.direction.z * ray.direction.y * origin.z * origin.x;
            c += self.c112 * ray.direction.x * ray.direction.y * origin.z.powi(2);
            d += self.c112 * 2.0 * ray.direction.z * origin.z * origin.x * origin.y;
            d += self.c112 * ray.direction.x * origin.z.powi(2) * origin.y;
            d += self.c112 * ray.direction.y * origin.z.powi(2) * origin.x;
            e += self.c112 * origin.z.powi(2) * origin.x * origin.y;
            // c220
            a += self.c220 * ray.direction.x.powi(2) * ray.direction.y.powi(2);
            b += self.c220 * 2.0 * ray.direction.x.powi(2) * ray.direction.y * origin.y;
            b += self.c220 * 2.0 * ray.direction.x * ray.direction.y.powi(2) * origin.x;
            c += self.c220 * ray.direction.x.powi(2) * origin.y.powi(2);
            c += self.c220 * 4.0 * ray.direction.x * ray.direction.y * origin.x * origin.y;
            c += self.c220 * ray.direction.y.powi(2) * origin.x.powi(2);
            d += self.c220 * 2.0 * ray.direction.x * origin.x * origin.y.powi(2);
            d += self.c220 * 2.0 * ray.direction.y * origin.x.powi(2) * origin.y;
            e += self.c220 * origin.x.powi(2) * origin.y.powi(2);
            // c022
            a += self.c022 * ray.direction.z.powi(2) * ray.direction.y.powi(2);
            b += self.c022 * 2.0 * ray.direction.z.powi(2) * ray.direction.y * origin.y;
            b += self.c022 * 2.0 * ray.direction.z * ray.direction.y.powi(2) * origin.z;
            c += self.c022 * ray.direction.z.powi(2) * origin.y.powi(2);
            c += self.c022 * 4.0 * ray.direction.z * ray.direction.y * origin.z * origin.y;
            c += self.c022 * ray.direction.y.powi(2) * origin.z.powi(2);
            d += self.c022 * 2.0 * ray.direction.z * origin.z * origin.y.powi(2);
            d += self.c022 * 2.0 * ray.direction.y * origin.z.powi(2) * origin.y;
            e += self.c022 * origin.z.powi(2) * origin.y.powi(2);
            // c202
            a += self.c202 * ray.direction.z.powi(2) * ray.direction.x.powi(2);
            b += self.c202 * 2.0 * ray.direction.z.powi(2) * ray.direction.x * origin.x;
            b += self.c202 * 2.0 * ray.direction.z * ray.direction.x.powi(2) * origin.z;
            c += self.c202 * ray.direction.z.powi(2) * origin.x.powi(2);
            c += self.c202 * 4.0 * ray.direction.z * ray.direction.x * origin.z * origin.x;
            c += self.c202 * ray.direction.x.powi(2) * origin.z.powi(2);
            d += self.c202 * 2.0 * ray.direction.z * origin.z * origin.x.powi(2);
            d += self.c202 * 2.0 * ray.direction.x * origin.z.powi(2) * origin.x;
            e += self.c202 * origin.z.powi(2) * origin.x.powi(2);
            // c300
            b += self.c300 * ray.direction.x.powi(3);
            c += self.c300 * 3.0 * ray.direction.x.powi(2) * origin.x;
            d += self.c300 * 3.0 * ray.direction.x * origin.x.powi(2);
            e += self.c300 * origin.x.powi(3);
            // c030
            b += self.c030 * ray.direction.y.powi(3);
            c += self.c030 * 3.0 * ray.direction.y.powi(2) * origin.y;
            d += self.c030 * 3.0 * ray.direction.y * origin.y.powi(2);
            e += self.c030 * origin.y.powi(3);
            // c003
            b += self.c003 * ray.direction.z.powi(3);
            c += self.c003 * 3.0 * ray.direction.z.powi(2) * origin.z;
            d += self.c003 * 3.0 * ray.direction.z * origin.z.powi(2);
            e += self.c003 * origin.z.powi(3);
            // c210
            b += self.c210 * ray.direction.x.powi(2) * ray.direction.y;
            c += self.c210 * ray.direction.x.powi(2) * origin.y;
            c += self.c210 * 2.0 * ray.direction.x * ray.direction.y * origin.x;
            d += self.c210 * 2.0 * ray.direction.x * origin.x * origin.y;
            d += self.c210 * ray.direction.y * origin.x.powi(2);
            e += self.c210 * origin.x.powi(2) * origin.y;
            // c201
            b += self.c201 * ray.direction.x.powi(2) * ray.direction.z;
            c += self.c201 * ray.direction.x.powi(2) * origin.z;
            c += self.c201 * 2.0 * ray.direction.x * ray.direction.z * origin.x;
            d += self.c201 * 2.0 * ray.direction.x * origin.x * origin.z;
            d += self.c201 * ray.direction.z * origin.x.powi(2);
            e += self.c201 * origin.x.powi(2) * origin.z;
            // c120
            b += self.c120 * ray.direction.y.powi(2) * ray.direction.x;
            c += self.c120 * ray.direction.y.powi(2) * origin.x;
            c += self.c120 * 2.0 * ray.direction.y * ray.direction.x * origin.y;
            d += self.c120 * 2.0 * ray.direction.y * origin.y * origin.x;
            d += self.c120 * ray.direction.x * origin.y.powi(2);
            e += self.c120 * origin.y.powi(2) * origin.x;
            // c021
            b += self.c021 * ray.direction.y.powi(2) * ray.direction.z;
            c += self.c021 * ray.direction.y.powi(2) * origin.z;
            c += self.c021 * 2.0 * ray.direction.y * ray.direction.z * origin.y;
            d += self.c021 * 2.0 * ray.direction.y * origin.y * origin.z;
            d += self.c021 * ray.direction.z * origin.y.powi(2);
            e += self.c021 * origin.y.powi(2) * origin.z;
            // c102
            b += self.c102 * ray.direction.z.powi(2) * ray.direction.x;
            c += self.c102 * ray.direction.z.powi(2) * origin.x;
            c += self.c102 * 2.0 * ray.direction.z * ray.direction.x * origin.z;
            d += self.c102 * 2.0 * ray.direction.z * origin.z * origin.x;
            d += self.c102 * ray.direction.x * origin.z.powi(2);
            e += self.c102 * origin.z.powi(2) * origin.x;
            // c012
            b += self.c012 * ray.direction.z.powi(2) * ray.direction.y;
            c += self.c012 * ray.direction.z.powi(2) * origin.y;
            c += self.c012 * 2.0 * ray.direction.z * ray.direction.y * origin.z;
            d += self.c012 * 2.0 * ray.direction.z * origin.z * origin.y;
            d += self.c012 * ray.direction.y * origin.z.powi(2);
            e += self.c012 * origin.z.powi(2) * origin.y;
            // c111
            b += self.c111 * ray.direction.x * ray.direction.y * ray.direction.z;
            c += self.c111 * ray.direction.x * ray.direction.y * origin.z;
            c += self.c111 * ray.direction.x * origin.y * ray.direction.z;
            c += self.c111 * origin.x * ray.direction.y * ray.direction.z;
            d += self.c111 * ray.direction.x * origin.y * origin.z;
            d += self.c111 * origin.x * ray.direction.y * origin.z;
            d += self.c111 * origin.x * origin.y * ray.direction.z;
            e += self.c111 * origin.x * origin.y * origin.z;
            // c200
            c += self.c200 * ray.direction.x.powi(2);
            d += self.c200 * 2.0 * ray.direction.x * origin.x;
            e += self.c200 * origin.x.powi(2);
            // c020
            c += self.c020 * ray.direction.y.powi(2);
            d += self.c020 * 2.0 * ray.direction.y * origin.y;
            e += self.c020 * origin.y.powi(2);
            // c002
            c += self.c002 * ray.direction.z.powi(2);
            d += self.c002 * 2.0 * ray.direction.z * origin.z;
            e += self.c002 * origin.z.powi(2);
            // c110
            c += self.c110 * ray.direction.x * ray.direction.y;
            d += self.c110 * ray.direction.x * origin.y;
            d += self.c110 * origin.x * ray.direction.y;
            e += self.c110 * origin.x * origin.y;
            // c011
            c += self.c011 * ray.direction.y * ray.direction.z;
            d += self.c011 * ray.direction.y * origin.z;
            d += self.c011 * origin.y * ray.direction.z;
            e += self.c011 * origin.y * origin.z;
            // c101
            c += self.c101 * ray.direction.x * ray.direction.z;
            d += self.c101 * ray.direction.x * origin.z;
            d += self.c101 * origin.x * ray.direction.z;
            e += self.c101 * origin.x * origin.z;
            // c100
            d += self.c100 * ray.direction.x;
            e += self.c100 * origin.x;
            // c010
            d += self.c010 * ray.direction.y;
            e += self.c010 * origin.y;
            // c001
            d += self.c001 * ray.direction.z;
            e += self.c001 * origin.z;
            // c000
            e += self.c000;
            // done
            (a, b, c, d, e)
        };

        quartic_roots(a, b, c, d, e)
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
            4.0 * self.c400 * position.x.powi(3)
                + 3.0 * self.c310 * position.x.powi(2) * position.y
                + 3.0 * self.c301 * position.x.powi(2) * position.z
                + 3.0 * self.c300 * position.x.powi(2)
                + 2.0 * self.c220 * position.x * position.y.powi(2)
                + 2.0 * self.c202 * position.x * position.z.powi(2)
                + 2.0 * self.c211 * position.x * position.y * position.z
                + 2.0 * self.c210 * position.x * position.y
                + 2.0 * self.c201 * position.x * position.z
                + 2.0 * self.c200 * position.x
                + self.c130 * position.y.powi(3)
                + self.c103 * position.z.powi(3)
                + self.c121 * position.y.powi(2) * position.z
                + self.c112 * position.y * position.z.powi(2)
                + self.c120 * position.y.powi(2)
                + self.c102 * position.z.powi(2)
                + self.c111 * position.y * position.z
                + self.c110 * position.y
                + self.c101 * position.z
                + self.c100,
            4.0 * self.c040 * position.y.powi(3)
                + 3.0 * self.c130 * position.y.powi(2) * position.x
                + 3.0 * self.c031 * position.y.powi(2) * position.z
                + 3.0 * self.c030 * position.y.powi(2)
                + 2.0 * self.c220 * position.y * position.x.powi(2)
                + 2.0 * self.c022 * position.y * position.z.powi(2)
                + 2.0 * self.c121 * position.y * position.x * position.z
                + 2.0 * self.c120 * position.y * position.x
                + 2.0 * self.c021 * position.y * position.z
                + 2.0 * self.c020 * position.y
                + self.c310 * position.x.powi(3)
                + self.c013 * position.z.powi(3)
                + self.c211 * position.x.powi(2) * position.z
                + self.c112 * position.x * position.z.powi(2)
                + self.c210 * position.x.powi(2)
                + self.c012 * position.z.powi(2)
                + self.c111 * position.x * position.z
                + self.c110 * position.x
                + self.c011 * position.z
                + self.c010,
            4.0 * self.c004 * position.z.powi(3)
                + 3.0 * self.c013 * position.z.powi(2) * position.y
                + 3.0 * self.c103 * position.z.powi(2) * position.x
                + 3.0 * self.c003 * position.z.powi(2)
                + 2.0 * self.c022 * position.z * position.y.powi(2)
                + 2.0 * self.c202 * position.z * position.x.powi(2)
                + 2.0 * self.c112 * position.z * position.y * position.x
                + 2.0 * self.c012 * position.z * position.y
                + 2.0 * self.c102 * position.z * position.x
                + 2.0 * self.c002 * position.z
                + self.c031 * position.y.powi(3)
                + self.c301 * position.x.powi(3)
                + self.c121 * position.y.powi(2) * position.x
                + self.c211 * position.y * position.x.powi(2)
                + self.c021 * position.y.powi(2)
                + self.c201 * position.x.powi(2)
                + self.c111 * position.y * position.x
                + self.c011 * position.y
                + self.c101 * position.x
                + self.c001,
        ))
    }
}

impl RTObject for Quartic {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let (inside_direction, inside_length) = (ray.origin - self.point).direction_and_length();
        let internal = self.internal_test(Ray {
            origin: self.point,
            direction: inside_direction,
        });
        let inside = (internal
            .into_iter()
            .filter(|hit| hit.distance < inside_length)
            .count()
            % 2
            == 0)
            == self.is_point_inside;

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
