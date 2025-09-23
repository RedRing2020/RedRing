use crate::model::geometry::geom2d::{
    point::Point2D,
    direction::Direction2D,
    line::Line2D,
    intersection_result::{IntersectionResult2D, IntersectionKind2D},
};

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

    pub fn intersection_with_line(&self, line: &Line2D) -> IntersectionResult2D {
        let line_result = line.intersection_with_line(&Line2D::new(self.origin, self.origin.add(&self.direction.to_vector())));
        if line_result.kind == IntersectionKind2D::None {
            return IntersectionResult2D {
                kind: IntersectionKind2D::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: EPSILON,
            };
        }

        let pt = &line_result.points[0];
        let v = pt.sub(&self.origin);
        let dot = v.dot(&self.direction.to_vector());

        if dot < -EPSILON {
            // 交点が Ray の逆方向にある
            return IntersectionResult2D {
                kind: IntersectionKind2D::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: EPSILON,
            };
        }

        let kind = if dot.abs() < EPSILON {
            IntersectionKind2D::Tangent
        } else {
            IntersectionKind2D::Point
        };

        IntersectionResult2D {
            kind,
            points: vec![*pt],
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn is_forward(&self, pt: &Point2D) -> bool {
        let v = pt.sub(&self.origin);
        v.dot(&self.direction.to_vector()) >= -EPSILON
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

    #[test]
    fn test_ray_line_intersection_forward() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0));
        let line = Line2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind2D::Point);
        assert_eq!(result.points.len(), 1);
        assert!((result.points[0].x - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_line_intersection_behind() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0));
        let line = Line2D::new(Point2D::new(-1.0, -1.0), Point2D::new(-1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind2D::None);
    }
}