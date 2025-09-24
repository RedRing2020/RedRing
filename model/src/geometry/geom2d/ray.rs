use crate::model::geometry::geom2d::{
    point::Point2,
    direction::Direction2,
    line::Line2,
    intersection_result::{IntersectionResult2, IntersectionKind2},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Ray2 {
    origin: Point2,
    direction: Direction2,
}

impl Ray2 {
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

    pub fn normal(&self) -> Direction2 {
        self.direction.normal()
    }

    pub fn intersection_with_line(&self, line: &Line2) -> IntersectionResult2 {
        let line_result = line.intersection_with_line(&Line2::new(self.origin, self.origin.add(&self.direction.to_vector())));
        if line_result.kind == IntersectionKind2::None {
            return IntersectionResult2 {
                kind: IntersectionKind2::None,
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
            return IntersectionResult2 {
                kind: IntersectionKind2::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: EPSILON,
            };
        }

        let kind = if dot.abs() < EPSILON {
            IntersectionKind2::Tangent
        } else {
            IntersectionKind2::Point
        };

        IntersectionResult2 {
            kind,
            points: vec![*pt],
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn is_forward(&self, pt: &Point2) -> bool {
        let v = pt.sub(&self.origin);
        v.dot(&self.direction.to_vector()) >= -EPSILON
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
        let ray = Ray2::new(origin, dir);
        let p = ray.evaluate(5.0);
        assert_eq!(p, Point2::new(5.0, 0.0));
    }

    #[test]
    fn test_normal() {
        let dir = Direction2::new(1.0, 0.0);
        let ray = Ray2::new(Point2::new(0.0, 0.0), dir);
        let normal = ray.normal();
        assert_eq!(normal, Direction2::new(0.0, 1.0));
    }

    #[test]
    fn test_ray_line_intersection_forward() {
        let ray = Ray2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0));
        let line = Line2::new(Point2::new(1.0, -1.0), Point2::new(1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind2::Point);
        assert_eq!(result.points.len(), 1);
        assert!((result.points[0].x - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_line_intersection_behind() {
        let ray = Ray2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0));
        let line = Line2::new(Point2::new(-1.0, -1.0), Point2::new(-1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind2::None);
    }
}