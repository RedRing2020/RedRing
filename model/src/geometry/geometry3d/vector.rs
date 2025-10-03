use std::ops::{Add, Sub, Mul, Neg};
use crate::geometry::geometry3d;
use crate::geometry::geometry3d::point::Point;
use crate::geometry_trait::normed::Normed;
use crate::geometry_trait::normalize::Normalize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }

    pub const ZERO: Vector = Vector { x: 0.0, y: 0.0, z: 0.0 };

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    pub fn between(a: &Point, b: &Point) -> Self {
        Self::new(b.x() - a.x(), b.y() - a.y(), b.z() - a.z())
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Vector) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, scalar: f64) -> Vector {
        Vector::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y, -self.z)
    }
}

impl Normed for geometry3d::vector::Vector {
    fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl Normalize for geometry3d::vector::Vector {
    fn normalize(&self) -> Self {
        let len = self.norm();
        if len == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }
}

// analysis クレートとの互換性のためのトレイト実装
impl analysis::NormedVector for Vector {
    fn norm(&self) -> f64 {
        self.norm()
    }
}
