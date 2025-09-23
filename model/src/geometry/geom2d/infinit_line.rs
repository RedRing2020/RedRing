use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine2D {
    origin: Point2D,
    direction: Direction2D,
}

impl InfiniteLine2D {
    pub fn new(origin: Point2D, direction: Direction2D) -> Self {
        Self { origin, direction }
    }

    pub fn origin(&self) -> &Point2D {
        &self.origin
    }

    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    pub fn evaluate(&self, t: f64) -> Point2D {
        self.origin.add(self.direction.x * t, self.direction.y * t)
    }

    pub fn distance_to_point(&self, point: &Point2D) -> f64 {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let normal = self.direction.normal();
        let dot = v[0] * normal.x + v[1] * normal.y;
        dot.abs()
    }

    pub fn contains_point(&self, point: &Point2D, epsilon: f64) -> bool {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let cross = v[0] * self.direction.y - v[1] * self.direction.x;
        cross.abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D};

    #[test]
    fn test_evaluate() {
        let origin = Point2D::new(0.0, 0.0);
        let dir = Direction2D::new(1.0, 0.0);
        let line = InfiniteLine2D::new(origin, dir);
        assert_eq!(line.evaluate(-2.0), Point2D::new(-2.0, 0.0));
        assert_eq!(line.evaluate(3.0), Point2D::new(3.0, 0.0));
    }

    #[test]
    fn test_distance_to_point() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0));
        let p = Point2D::new(2.0, 5.0);
        let d = line.distance_to_point(&p);
        assert!((d - 5.0).abs() < 1e-10);
    }
}