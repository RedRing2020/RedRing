use super::{point::Point2, direction::Direction2};

#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine2 {
    origin: Point2,
    direction: Direction2,
}

impl InfiniteLine2 {
    pub fn new(origin: Point2, direction: Direction2) -> Self {
        Self { origin, direction }
    }

    pub fn origin(&self) -> &Point2 {
        &self.origin
    }

    pub fn direction(&self) -> &Direction2 {
        &self.direction
    }

    pub fn evaluate(&self, t: f64) -> Point2 {
        self.origin.add(self.direction.x * t, self.direction.y * t)
    }

    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let normal = self.direction.normal();
        let dot = v[0] * normal.x + v[1] * normal.y;
        dot.abs()
    }

    pub fn contains_point(&self, point: &Point2, epsilon: f64) -> bool {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let cross = v[0] * self.direction.y - v[1] * self.direction.x;
        cross.abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2, direction::Direction2};

    #[test]
    fn test_evaluate() {
        let origin = Point2::new(0.0, 0.0);
        let dir = Direction2::new(1.0, 0.0);
        let line = InfiniteLine2::new(origin, dir);
        assert_eq!(line.evaluate(-2.0), Point2::new(-2.0, 0.0));
        assert_eq!(line.evaluate(3.0), Point2::new(3.0, 0.0));
    }

    #[test]
    fn test_distance_to_point() {
        let line = InfiniteLine2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0));
        let p = Point2::new(2.0, 5.0);
        let d = line.distance_to_point(&p);
        assert!((d - 5.0).abs() < 1e-10);
    }
}