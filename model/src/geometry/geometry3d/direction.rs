use crate::geometry_trait::Normalize;

use super::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction(Vector);

impl Direction {
    /// Create a Direction from a Vector. Returns None if the vector is zero or near-zero.
    pub fn from_vector(v: Vector) -> Option<Self> {
        let normalized = v.normalize();
        if normalized.norm() > 1e-10 {
            Some(Direction(normalized))
        } else {
            None
        }
    }

    /// Create a new Direction from components. Returns None if the vector is zero or near-zero.
    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {
        Self::from_vector(Vector::new(x, y, z))
    }

    /// Get the x component of the direction
    pub fn x(&self) -> f64 {
        self.0.x()
    }

    /// Get the y component of the direction
    pub fn y(&self) -> f64 {
        self.0.y()
    }

    /// Get the z component of the direction
    pub fn z(&self) -> f64 {
        self.0.z()
    }

    /// Get the length (norm) of the direction vector (should be ~1.0)
    pub fn length(&self) -> f64 {
        self.0.norm()
    }

    /// Normalize the direction (should already be normalized, but ensures it)
    pub fn normalize(&self) -> Option<Self> {
        Self::from_vector(self.0)
    }

    pub fn as_vector(&self) -> Vector {
        self.0
    }

    pub fn to_vector(&self) -> Vector {
        self.0
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }

    pub fn cross(&self, other: &Self) -> Option<Self> {
        let cross = self.0.cross(&other.0);
        Direction::from_vector(cross)
    }

    pub fn negate(&self) -> Self {
        Direction(Vector::new(-self.0.x(), -self.0.y(), -self.0.z()))
    }

    /// normalベクトルから円の直交基底(u, v)を生成
    pub fn orthonormal_basis(&self) -> (Vector, Vector) {
        let n = self.to_vector();
        // nと直交する任意のベクトルを選ぶ
        let up = if n.z().abs() < 0.99 {
            Vector::new(0.0, 0.0, 1.0)
        } else {
            Vector::new(0.0, 1.0, 0.0)
        };
        let u = n.cross(&up).normalize();
        let v = n.cross(&u).normalize();
        (u, v)
    }
}
