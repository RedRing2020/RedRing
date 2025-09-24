use crate::model::geometry::geom2d::{
    point::Point,
    direction::Direction,
    line::Line,
    intersection_result::{IntersectionResult, IntersectionKind},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    origin: Point,
    direction: Direction,
}

impl Ray {
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

    pub fn normal(&self) -> Direction {
        self.direction.normal()
    }

    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult {
        let line_result = line.intersection_with_line(&Line::new(self.origin, self.origin.add(&self.direction.to_vector())));
        if line_result.kind == IntersectionKind::None {
            return IntersectionResult {
                kind: IntersectionKind::None,
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
            return IntersectionResult {
                kind: IntersectionKind::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: EPSILON,
            };
        }

        let kind = if dot.abs() < EPSILON {
            IntersectionKind::Tangent
        } else {
            IntersectionKind::Point
        };

        IntersectionResult {
            kind,
            points: vec![*pt],
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn is_forward(&self, pt: &Point) -> bool {
        let v = pt.sub(&self.origin);
        v.dot(&self.direction.to_vector()) >= -EPSILON
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
        let ray = Ray::new(origin, dir);
        let p = ray.evaluate(5.0);
        assert_eq!(p, Point::new(5.0, 0.0));
    }

    #[test]
    fn test_normal() {
        let dir = Direction::new(1.0, 0.0);
        let ray = Ray::new(Point::new(0.0, 0.0), dir);
        let normal = ray.normal();
        assert_eq!(normal, Direction::new(0.0, 1.0));
    }

    #[test]
    fn test_ray_line_intersection_forward() {
        let ray = Ray::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0));
        let line = Line::new(Point::new(1.0, -1.0), Point::new(1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind::Point);
        assert_eq!(result.points.len(), 1);
        assert!((result.points[0].x - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_line_intersection_behind() {
        let ray = Ray::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0));
        let line = Line::new(Point::new(-1.0, -1.0), Point::new(-1.0, 1.0));
        let result = ray.intersection_with_line(&line);
        assert_eq!(result.kind, IntersectionKind::None);
    }
}