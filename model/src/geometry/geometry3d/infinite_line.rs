use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;

/// Represents an infinite line in 3D space.
/// Defined by an origin point and a direction.
#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine {
    origin: Point,
    direction: Direction,
}

impl InfiniteLine {
    /// Creates a new infinite line from an origin and a direction.
    pub fn new(origin: Point, direction: Direction) -> Self {
        Self { origin, direction }
    }

    /// Returns the origin point of the line.
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// Returns the direction of the line.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns a point on the line at parameter t ∈ ℝ.
    pub fn at(&self, t: f64) -> Point {
        self.origin.translate(&self.direction.as_vector().scale(t))
    }

    /// Computes the shortest distance from a point to the infinite line.
    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let v = Vector::between(&self.origin, point);
        let proj = self.direction.as_vector().dot(&v);
        let closest = self.at(proj);
        point.distance_to(&closest)
    }
}