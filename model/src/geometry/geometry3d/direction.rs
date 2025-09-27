use super::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction(Vector);

impl Direction {
    pub fn from_vector(v: Vector) -> Option<Self> {
        v.normalize().map(Direction)
    }

    pub fn as_vector(&self) -> Vector {
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
}