use jsonc::Value;

use crate::{ImageCache, ImageLoader};

use super::RTObject;

use core::types::rt::{Hit, Ray};
use std::collections::HashMap;

fn remove_duplicate_hits(sorted: &mut Vec<Hit>) {
    let mut result: Vec<Hit> = Vec::new();
    for hit in sorted.drain(..) {
        if let Some(last) = result.last() {
            if last.is_front_face != hit.is_front_face
                && (last.distance - hit.distance).abs() < 1e-6
            {
                result.pop();
                continue;
            }
        }
        result.push(hit);
    }
    *sorted = result;
}

struct Union {
    a: Box<dyn RTObject + Send + Sync>,
    b: Box<dyn RTObject + Send + Sync>,
}

impl RTObject for Union {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let mut a_hits = self.a.test(ray);
        let mut b_hits = self.b.test(ray);

        if a_hits.is_empty() {
            return b_hits;
        }
        if b_hits.is_empty() {
            return a_hits;
        }

        let mut all_hits = Vec::new();
        all_hits.append(&mut a_hits);
        all_hits.append(&mut b_hits);
        all_hits.sort_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

        remove_duplicate_hits(&mut all_hits);

        let mut stack = 0;
        let mut result = Vec::new();
        for hit in all_hits {
            if hit.is_front_face {
                if stack == 0 {
                    result.push(hit);
                }
                stack += 1;
            } else {
                stack -= 1;
                if stack == 0 {
                    result.push(hit);
                }
            }
        }

        result
    }
}

struct Intersection {
    a: Box<dyn RTObject + Send + Sync>,
    b: Box<dyn RTObject + Send + Sync>,
}

impl RTObject for Intersection {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let mut a_hits = self.a.test(ray);
        if a_hits.is_empty() {
            return a_hits;
        }

        let mut b_hits = self.b.test(ray);
        if b_hits.is_empty() {
            return b_hits;
        }

        let mut all_hits = Vec::new();
        all_hits.append(&mut a_hits);
        all_hits.append(&mut b_hits);
        all_hits.sort_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

        remove_duplicate_hits(&mut all_hits);

        let mut stack = 0;
        let mut result = Vec::new();
        for hit in all_hits {
            if hit.is_front_face {
                stack += 1;
                if stack == 2 {
                    result.push(hit);
                }
            } else {
                if stack == 2 {
                    result.push(hit);
                }
                stack -= 1;
            }
        }

        result
    }
}

struct Difference {
    a: Box<dyn RTObject + Send + Sync>,
    b: Box<dyn RTObject + Send + Sync>,
}

impl RTObject for Difference {
    fn test(&self, ray: Ray) -> Vec<Hit> {
        let mut a_hits = self.a.test(ray);
        if a_hits.is_empty() {
            return a_hits;
        }

        let mut b_hits = self.b.test(ray);
        if b_hits.is_empty() {
            return a_hits;
        }

        let mut all_hits = Vec::new();
        all_hits.append(&mut a_hits.clone());
        all_hits.append(&mut a_hits);
        all_hits.append(&mut b_hits);
        all_hits.sort_by(|h1, h2| h1.distance.partial_cmp(&h2.distance).unwrap());

        remove_duplicate_hits(&mut all_hits);

        let mut stack = 0;
        let mut is_front_face = false;
        let mut result = Vec::new();
        for hit in all_hits {
            let prev_stack = stack;
            if hit.is_front_face {
                stack += 1;
            } else {
                stack -= 1;
            }
            if prev_stack == 2 || stack == 2 {
                is_front_face = !is_front_face;
                result.push(if is_front_face == hit.is_front_face {
                    hit
                } else {
                    Hit {
                        is_front_face,
                        normal: -hit.normal,
                        ..hit
                    }
                });
            }
        }

        remove_duplicate_hits(&mut result);

        result
    }
}

pub fn from_json_value(
    dict: &HashMap<String, Value>,
    type_str: &String,
    image_cache: &ImageCache<impl ImageLoader>,
) -> Result<Box<dyn RTObject + Send + Sync>, String> {
    let a = crate::object::from_json_value(
        dict.get("a").ok_or("Missing required field: a")?,
        image_cache,
    )?;
    let b = crate::object::from_json_value(
        dict.get("b").ok_or("Missing required field: b")?,
        image_cache,
    )?;

    match type_str.as_str() {
        "union" => Ok(Box::new(Union { a, b })),
        "intersection" => Ok(Box::new(Intersection { a, b })),
        "difference" => Ok(Box::new(Difference { a, b })),
        _ => Err(format!("Unknown csg type: {}", type_str)),
    }
}
