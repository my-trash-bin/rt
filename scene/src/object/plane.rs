use crate::{
    object::{material_from_json_value, quadratic::Quadratic, quadric::Quadric, quartic::Quartic},
    position_from_json_value, ImageCache, ImageLoader,
};

use super::RTObject;
use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Hit, Ray},
};
use jsonc::Value;
use std::collections::HashMap;
use types::LDRColor;

#[derive(Clone, Debug)]
pub struct Plane {
    pub position: Position,
    pub albedo: LDRColor,
    pub roughness: f64,
    pub metallic: f64,

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

impl RTObject for Plane {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let origin: Position = (ray.origin - self.position).into();

        let (a, b) = {
            let mut a = 0.0;
            let mut b = 0.0;

            // c100
            a += self.c100 * ray.direction.x;
            b += self.c100 * origin.x;
            // c010
            a += self.c010 * ray.direction.y;
            b += self.c010 * origin.y;
            // c001
            a += self.c001 * ray.direction.z;
            b += self.c001 * origin.z;
            // c000
            b += self.c000;
            // done
            (a, b)
        };

        linear_roots(a, b)
            .into_iter()
            .filter(|t| *t >= 0.0)
            .map(|distance| Hit {
                distance,
                normal: normal(self),
                albedo: self.albedo,
                is_front_face: true, // decided later
                roughness: self.roughness,
                metallic: self.metallic,
            })
            .collect()
    }
}

fn normal(thiz: &Plane) -> Direction {
    Direction::new(Vec3::new(thiz.c100, thiz.c010, thiz.c001))
}

pub fn from_json_value(
    dict: &HashMap<String, Value>,
    image_cache: &ImageCache<impl ImageLoader>,
) -> Result<Box<dyn RTObject + Send + Sync>, String> {
    let position = dict
        .get("position")
        .map(position_from_json_value)
        .unwrap_or(Ok(Position::new(Vec3::ZERO)))?;
    let (albedo, roughness, metallic) =
        material_from_json_value(dict.get("material"), image_cache)?;
    let point =
        position_from_json_value(dict.get("point").ok_or("Missing required field: point")?)?;
    let is_point_inside = dict
        .get("isPointInside")
        .ok_or("Missing required field: isPointInside")?;
    let Value::Bool(is_point_inside) = is_point_inside else {
        return Err("is_point_inside must be a boolean".to_string());
    };
    let coefficients = dict
        .get("coefficients")
        .ok_or("Missing required field: coefficients")?;
    let Value::Object(dict) = coefficients else {
        return Err("Coefficients must be a JSON object".to_string());
    };
    let c400 = dict
        .get("x^4")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c040 = dict
        .get("y^4")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c004 = dict
        .get("z^4")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c310 = dict
        .get("x^3y")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c301 = dict
        .get("x^3z")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c130 = dict
        .get("xy^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c031 = dict
        .get("y^3z")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c103 = dict
        .get("xz^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c013 = dict
        .get("yz^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c211 = dict
        .get("x^2yz")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c121 = dict
        .get("xy^2z")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c112 = dict
        .get("xyz^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c220 = dict
        .get("x^2y^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c022 = dict
        .get("y^2z^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c202 = dict
        .get("x^2z^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c300 = dict
        .get("x^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c030 = dict
        .get("y^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c003 = dict
        .get("z^3")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c210 = dict
        .get("x^2y")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c201 = dict
        .get("x^2z")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c120 = dict
        .get("xy^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c021 = dict
        .get("y^2z")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c102 = dict
        .get("xz^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c012 = dict
        .get("yz^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c111 = dict
        .get("xyz")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c200 = dict
        .get("x^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c020 = dict
        .get("y^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c002 = dict
        .get("z^2")
        .map(f64_from_json_value)
        .unwrap_or(Ok(0.0))?;
    let c110 = dict.get("xy").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c011 = dict.get("yz").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c101 = dict.get("xz").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c100 = dict.get("x").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c010 = dict.get("y").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c001 = dict.get("z").map(f64_from_json_value).unwrap_or(Ok(0.0))?;
    let c000 = dict.get("0").map(f64_from_json_value).unwrap_or(Ok(0.0))?;

    if c400 != 0.0
        || c040 != 0.0
        || c004 != 0.0
        || c310 != 0.0
        || c301 != 0.0
        || c130 != 0.0
        || c031 != 0.0
        || c103 != 0.0
        || c013 != 0.0
        || c211 != 0.0
        || c121 != 0.0
        || c112 != 0.0
        || c220 != 0.0
        || c022 != 0.0
        || c202 != 0.0
    {
        Ok(Box::new(Quartic {
            position,
            albedo,
            roughness,
            metallic,
            c400,
            c040,
            c004,
            c310,
            c301,
            c130,
            c031,
            c103,
            c013,
            c211,
            c121,
            c112,
            c220,
            c022,
            c202,
            c300,
            c030,
            c003,
            c210,
            c201,
            c120,
            c021,
            c102,
            c012,
            c111,
            c200,
            c020,
            c002,
            c110,
            c011,
            c101,
            c100,
            c010,
            c001,
            c000,
            point,
            is_point_inside: *is_point_inside,
        }))
    } else if c300 != 0.0
        || c030 != 0.0
        || c003 != 0.0
        || c210 != 0.0
        || c201 != 0.0
        || c120 != 0.0
        || c021 != 0.0
        || c102 != 0.0
        || c012 != 0.0
        || c111 != 0.0
    {
        Ok(Box::new(Quadratic {
            position,
            albedo,
            roughness,
            metallic,
            c300,
            c030,
            c003,
            c210,
            c201,
            c120,
            c021,
            c102,
            c012,
            c111,
            c200,
            c020,
            c002,
            c110,
            c011,
            c101,
            c100,
            c010,
            c001,
            c000,
            point,
            is_point_inside: *is_point_inside,
        }))
    } else if c200 != 0.0 || c020 != 0.0 || c002 != 0.0 || c110 != 0.0 || c011 != 0.0 || c101 != 0.0
    {
        Ok(Box::new(Quadric {
            position,
            albedo,
            roughness,
            metallic,
            c200,
            c020,
            c002,
            c110,
            c011,
            c101,
            c100,
            c010,
            c001,
            c000,
            point,
            is_point_inside: *is_point_inside,
        }))
    } else {
        Ok(Box::new(Plane {
            position,
            albedo,
            roughness,
            metallic,
            c100,
            c010,
            c001,
            c000,
            point,
            is_point_inside: *is_point_inside,
        }))
    }
}

fn f64_from_json_value(json: &Value) -> Result<f64, String> {
    let Value::Number(number) = json else {
        return Err("Coefficient must be a number".to_string());
    };
    Ok(*number)
}
