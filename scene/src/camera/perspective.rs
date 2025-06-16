use core::types::{
    math::{Direction, Position, Vec3},
    rt::{Camera, Ray},
};

#[derive(Clone, Debug)]
pub struct AspectRatio {
    aspect_ratio: f64,
}

#[derive(Clone, Debug)]
pub enum FovMode {
    X,
    Y,
    Cover(AspectRatio),
    Contain(AspectRatio),
}

fn forward() -> Direction {
    Direction::new(Vec3::Y)
}

#[derive(Clone, Debug)]
pub struct DeserializablePerspectiveCamera {
    fov: f64,
    fov_mode: FovMode,
    position: Position,
    direction: Direction,
}

impl DeserializablePerspectiveCamera {
    pub fn into_camera(self, screen_aspect_ratio: f64) -> Box<dyn Camera + Send + Sync> {
        let (tan_half_fov_x, tan_half_fov_y) = match self.fov_mode {
            FovMode::X => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / screen_aspect_ratio;
                (tan_half_fov_x, tan_half_fov_y)
            }
            FovMode::Y => {
                let tan_half_fov_y = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_x = tan_half_fov_y * screen_aspect_ratio;
                (tan_half_fov_x, tan_half_fov_y)
            }
            FovMode::Cover(aspect_ratio) => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / aspect_ratio.aspect_ratio;
                let scale = (screen_aspect_ratio / aspect_ratio.aspect_ratio).max(1.0);
                (tan_half_fov_x * scale, tan_half_fov_y * scale)
            }
            FovMode::Contain(aspect_ratio) => {
                let tan_half_fov_x = (self.fov.to_radians() / 2.0).tan();
                let tan_half_fov_y = tan_half_fov_x / aspect_ratio.aspect_ratio;
                let scale = (screen_aspect_ratio / aspect_ratio.aspect_ratio).min(1.0);
                (tan_half_fov_x * scale, tan_half_fov_y * scale)
            }
        };

        let right = self.direction.cross(Vec3::Z).normalize();
        let up = right.cross(*self.direction).normalize();
        Box::new(PerspectiveCamera {
            tan_half_fov_x,
            tan_half_fov_y,
            position: self.position,
            direction: self.direction,
            right,
            up,
        })
    }
}

struct PerspectiveCamera {
    tan_half_fov_x: f64,
    tan_half_fov_y: f64,
    position: Position,
    direction: Direction,
    right: Vec3,
    up: Vec3,
}

impl Camera for PerspectiveCamera {
    fn ray(&self, x: f64, y: f64) -> Ray {
        let dir_x = (2.0 * x - 1.0) * self.tan_half_fov_x;
        let dir_z = (1.0 - 2.0 * y) * self.tan_half_fov_y;

        let direction = Direction::new(*self.direction + dir_x * self.right + self.up * dir_z);

        Ray {
            origin: self.position,
            direction,
        }
    }
}
