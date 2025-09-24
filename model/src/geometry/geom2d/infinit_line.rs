use super::{point::Point, direction::Direction};
use super::kind::CurveKind2;
use super::curve::curve_trait::Curve2;

#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine {
    origin: Point,
    direction: Direction,
}

impl Curve2 for InfiniteLine {
    fn kind(&self) -> CurveKind2 {
        CurveKind2::InfiniteLine
    }
}

impl InfiniteLine {
    pub fn new(origin: Point, direction: Direction) -> Self {
        Self { origin, direction }
    }

    pub fn origin(&self) -> &Point {
        &self.origin
    }

    pub fn direction(&self) -> &Direction {
        &self.direction
    }

    pub fn evaluate(&self, t: f64) -> Point {
        self.origin.add(self.direction.x * t, self.direction.y * t)
    }

    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let normal = self.direction.normal();
        let dot = v[0] * normal.x + v[1] * normal.y;
        dot.abs()
    }

    pub fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let cross = v[0] * self.direction.y - v[1] * self.direction.x;
        cross.abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point, direction::Direction};

    #[test]
    fn test_evaluate() {
        let origin = Point::new(0.0, 0.0);
        let dir = Direction::new(1.0, 0.0);
        let line = InfiniteLine::new(origin, dir);
        assert_eq!(line.evaluate(-2.0), Point::new(-2.0, 0.0));
        assert_eq!(line.evaluate(3.0), Point::new(3.0, 0.0));
    }

    #[test]
    fn test_distance_to_point() {
        let line = InfiniteLine::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0));
        let p = Point::new(2.0, 5.0);
        let d = line.distance_to_point(&p);
        assert!((d - 5.0).abs() < 1e-10);
    }
}