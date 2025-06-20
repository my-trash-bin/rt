use crate::types::{
    math::Direction,
    rt::{Ray, Scene},
};
use ::types::{HDRColor, LDRColor};

pub mod types;

pub fn sample(scene: &Scene, x: f64, y: f64) -> HDRColor {
    let ray = scene.camera.ray(x, y);
    if let Some(hit) = scene.test(ray) {
        let position = ray.origin + ray.direction * hit.distance + hit.normal * 1e-3;
        let mut result = scene.ambient_light * hit.albedo;
        for light in scene.lights.iter() {
            if let Some((color, direction, distance)) = light.test(position) {
                let shadow_ray = Ray {
                    origin: position,
                    direction,
                };

                let shadow_hit = scene.test(shadow_ray);

                let is_shadowed = if distance.is_finite() {
                    shadow_hit.map(|x| x.distance).unwrap_or(f64::INFINITY) < distance
                } else {
                    shadow_hit.is_some()
                };

                if !is_shadowed {
                    result = result
                        + brdf(
                            -ray.direction,
                            direction,
                            hit.normal,
                            hit.roughness,
                            hit.metallic,
                            hit.albedo,
                            color,
                        )
                }
            }
        }
        result
    } else {
        (scene.sky_color)(ray.direction)
    }
}

fn brdf(
    surface_to_view: Direction,
    surface_to_light: Direction,
    surface_normal: Direction,
    roughness: f64,
    metallic: f64,
    albedo: LDRColor,
    light_color: HDRColor,
) -> HDRColor {
    fn fresnel_schlick(cos_theta: f64, f0: f64) -> f64 {
        let cos_theta = cos_theta.clamp(0.0, 1.0);
        f0 + (1.0 - f0) * (1.0 - cos_theta).powf(5.0)
    }

    fn ggx_ndf(n: Direction, h: Direction, roughness: f64) -> f64 {
        let alpha = roughness * roughness;
        let alpha2 = alpha * alpha;
        let cos_n_h = n.dot(h).clamp(0.0, 1.0);
        let cos_n_h2 = cos_n_h * cos_n_h;
        let denom = cos_n_h2 * alpha2 + (1.0 - cos_n_h2);
        alpha2 / (std::f64::consts::PI * denom * denom)
    }

    fn geometric_attenuation(n: Direction, v: Direction, l: Direction, roughness: f64) -> f64 {
        let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
        let cos_n_v = n.dot(v).max(1e-5);
        let g_v = cos_n_v / (cos_n_v * (1.0 - k) + k);
        let cos_n_l = n.dot(l).max(1e-5);
        let g_l = cos_n_l / (cos_n_l * (1.0 - k) + k);
        g_v * g_l
    }

    let n_dot_l = surface_normal.dot(surface_to_light).max(0.0);

    let f0 = LDRColor {
        r: albedo.r * metallic + (1.0 - metallic) * 0.04,
        g: albedo.g * metallic + (1.0 - metallic) * 0.04,
        b: albedo.b * metallic + (1.0 - metallic) * 0.04,
    };

    let h = Direction::new(*surface_to_view + *surface_to_light);
    let d = ggx_ndf(surface_normal, h, roughness);
    let g = geometric_attenuation(surface_normal, surface_to_view, surface_to_light, roughness);
    let h_dot_v = h.dot(surface_to_view).clamp(0.0, 1.0);

    let spec_common =
        (d * g) / (4.0 * surface_normal.dot(surface_to_view).max(1e-5) * n_dot_l.max(1e-5));

    let fresnel_r = fresnel_schlick(h_dot_v, f0.r);
    let fresnel_g = fresnel_schlick(h_dot_v, f0.g);
    let fresnel_b = fresnel_schlick(h_dot_v, f0.b);

    let specular = LDRColor {
        r: spec_common * fresnel_r,
        g: spec_common * fresnel_g,
        b: spec_common * fresnel_b,
    };

    let kd = LDRColor {
        r: (1.0 - fresnel_r) * (1.0 - metallic),
        g: (1.0 - fresnel_g) * (1.0 - metallic),
        b: (1.0 - fresnel_b) * (1.0 - metallic),
    };

    let diffuse = LDRColor {
        r: kd.r * albedo.r / std::f64::consts::PI,
        g: kd.g * albedo.g / std::f64::consts::PI,
        b: kd.b * albedo.b / std::f64::consts::PI,
    };

    HDRColor {
        r: (diffuse.r + specular.r) * light_color.r * n_dot_l,
        g: (diffuse.g + specular.g) * light_color.g * n_dot_l,
        b: (diffuse.b + specular.b) * light_color.b * n_dot_l,
    }
}
