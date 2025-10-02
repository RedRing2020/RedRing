use crate::geometry_trait::Normalize;

use super::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction(Vector);

impl Direction {
    pub fn from_vector(v: Vector) -> Option<Self> {
        let len = v.norm();
        if len == 0.0 {
            None
        } else {
            Some(Direction(v.normalize()))
        }
    }

    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len > 0.0 {
            Some(Self::new(self.x() / len, self.y() / len, self.z() / len))
        } else {
            None
        }
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
