use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;

/// Represents a half-infinite ray in 3D space.
/// Defined by a start point and a direction.
#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    start: Point,
    direction: Direction,
}

impl Ray {
    /// Creates a new ray from a start point and a direction.
    pub fn new(start: Point, direction: Direction) -> Self {
        Self { start, direction }
    }

    /// Returns the start point of the ray.
    pub fn start(&self) -> Point {
        self.start
    }

    /// Returns the direction of the ray.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns a point on the ray at parameter t ≥ 0.
    pub fn at(&self, t: f64) -> Option<Point> {
        if t < 0.0 {
            None
        } else {
            Some(self.start.translate(&self.direction.as_vector().scale(t)))
        }
    }

    /// Computes the shortest distance from a point to the ray.
    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let v = Vector::between(&self.start, point);
        let proj = self.direction.as_vector().dot(&v);

        let closest = if proj < 0.0 {
            self.start
        } else {
            self.at(proj).unwrap()
        };

        point.distance_to(&closest)
    }
}