use std::ops::{Add, Div, Mul};

#[derive(Clone, Copy, Debug)]
pub struct HDRColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LDRColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl HDRColor {
    pub const BLACK: HDRColor = HDRColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
}

impl Default for LDRColor {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

impl Default for HDRColor {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

impl LDRColor {
    pub fn new(r: f64, g: f64, b: f64) -> LDRColor {
        LDRColor {
            r: r.min(1.0),
            g: g.min(1.0),
            b: b.min(1.0),
        }
    }
}

impl Mul<LDRColor> for HDRColor {
    type Output = HDRColor;

    fn mul(self, rhs: LDRColor) -> Self::Output {
        HDRColor {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<HDRColor> for HDRColor {
    type Output = HDRColor;

    fn mul(self, rhs: HDRColor) -> Self::Output {
        HDRColor {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<HDRColor> for LDRColor {
    type Output = HDRColor;

    fn mul(self, rhs: HDRColor) -> Self::Output {
        HDRColor {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Add for HDRColor {
    type Output = HDRColor;

    fn add(self, rhs: Self) -> Self::Output {
        HDRColor {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Mul<f64> for HDRColor {
    type Output = HDRColor;

    fn mul(self, rhs: f64) -> Self::Output {
        HDRColor {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Div<f64> for HDRColor {
    type Output = HDRColor;

    fn div(self, rhs: f64) -> Self::Output {
        HDRColor {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}
