use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Ray2D {
    origin: Point2D,
    direction: Direction2D,
}

impl Ray2D {
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

    pub fn normal(&self) -> Direction2D {
        self.direction.normal()
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
        let ray = Ray2D::new(origin, dir);
        let p = ray.evaluate(5.0);
        assert_eq!(p, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_normal() {
        let dir = Direction2D::new(1.0, 0.0);
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), dir);
        let normal = ray.normal();
        assert_eq!(normal, Direction2D::new(0.0, 1.0));
    }
}