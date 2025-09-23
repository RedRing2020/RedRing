use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Line2D {
    origin: Point2D,
    direction: Direction2D,
    length: f64,
}

// 公開APIは必要最小限に限定
impl Line2D {
    pub fn new(origin: Point2D, direction: Direction2D, length: f64) -> Self {
        Self { origin, direction, length }
    }

    pub fn origin(&self) -> &Point2D {
        &self.origin
    }

    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn set_length(&mut self, new_length: f64) {
        self.length = new_length.max(0.0);
    }

    pub fn evaluate(&self, t: f64) -> Point2D {
        self.origin.add(self.direction.x * self.length * t, self.direction.y * self.length * t)
    }

    pub fn end_point(&self) -> Point2D {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point2D {
        self.evaluate(0.5)
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
        let line = Line2D::new(origin, dir, 10.0);
        let p = line.evaluate(0.5);
        assert_eq!(p, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_end_point() {
        let origin = Point2D::new(1.0, 2.0);
        let dir = Direction2D::new(0.0, 1.0);
        let line = Line2D::new(origin, dir, 3.0);
        let end = line.end_point();
        assert_eq!(end, Point2D::new(1.0, 5.0));
    }

    #[test]
    fn test_midpoint() {
        let origin = Point2D::new(0.0, 0.0);
        let dir = Direction2D::new(1.0, 0.0);
        let line = Line2D::new(origin, dir, 10.0);
        let mid = line.midpoint();
        assert_eq!(mid, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_from_points() {
        let start = Point2D::new(2.0, 2.0);
        let end = Point2D::new(6.0, 2.0);
        let line = Line2D::from_points(start, end);
        assert_eq!(line.origin, start);
        assert_eq!(line.end_point(), end);
        assert_eq!(line.length, 4.0);
        assert_eq!(line.direction, Direction2D::new(1.0, 0.0));
    }
}