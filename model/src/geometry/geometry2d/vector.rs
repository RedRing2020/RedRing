use std::ops::{Add, Sub, Mul, Neg};
use crate::geometry::geometry2d;
use crate::geometry::geometry2d::direction::Direction;
use crate::geometry_trait::normed::Normed;
use crate::geometry_trait::normalize::Normalize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    /// x成分とy成分からベクトルを構築
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 原点ベクトル
    pub const ZERO: Vector = Vector { x: 0.0, y: 0.0 };

    /// ベクトルの長さ（ノルム）
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 内積
    pub fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 外積（スカラー値）
    pub fn cross(&self, other: &Vector) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// 方向ベクトルから角度（ラジアン）を取得
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn to_direction(&self) -> Direction {
        Direction::new(self.x, self.y)
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Vector) -> Vector {
        Vector { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, scalar: f64) -> Vector {
        Vector { x: self.x * scalar, y: self.y * scalar }
    }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector { x: -self.x, y: -self.y }
    }
}

impl Normed for geometry2d::vector::Vector {
    fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
impl Normalize for geometry2d::vector::Vector {
    fn normalize(&self) -> Self {
        let len = self.norm();
        if len == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }
}
