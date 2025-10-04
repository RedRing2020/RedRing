use crate::geometry::geometry2d::{
    point::Point,
    direction::Direction,
};

use crate::geometry_trait::point_ops::PointOps;



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
    
    pub fn contains_point(&self, _pt: &Point, _epsilon: f64) -> bool { false }

    pub fn parameter_of(&self, _pt: &Point) -> f64 { 0.0 }

    pub fn is_forward(&self, pt: &Point, epsilon: f64) -> bool {
        let v = pt.sub(&self.origin);
        v.to_vector().dot(&self.direction.to_vector()) >= -epsilon
    }
/*
    pub fn intersection_with_line(&self, line: &Line, epsilon: f64) -> IntersectionResult<Point> {
        // Rayの方向に十分伸ばした線分を作る
        let ray_end = self.origin().add(self.direction().to_point());
        let ray_line = Line::new(self.origin, ray_end);

        // Line同士の交点を計算
        let line_result = line.intersection_with_line(&ray_line, epsilon);

        if line_result.kind == IntersectionKind::None {
            return IntersectionResult {
                kind: IntersectionKind::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: epsilon,
            };
        }

        let pt = &line_result.points[0];
        if !self.is_forward(pt, epsilon) {
            // 交点がRayの逆方向
            return IntersectionResult {
                kind: IntersectionKind::None,
                points: vec![],
                parameters: vec![],
                tolerance_used: epsilon,
            };
        }

        let kind = if (pt.sub(&self.origin).to_vector().dot(&self.direction.to_vector())).abs() < epsilon {
            IntersectionKind::Tangent
        } else {
            IntersectionKind::Point
        };

        IntersectionResult {
            kind,
            points: vec![*pt],
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    pub fn intersection_with_ellipse(&self, ellipse: &Ellipse, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| ellipse.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| *a.distance_to(*b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

        let kind = match points.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points,
            parameters,
            tolerance_used: epsilon,
        }
    }
*/
}
/*
impl Curve2D for Ray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Ray
    }

    fn evaluate(&self, t: f64) -> Point {
        self.origin.add(self.direction.x() * t, self.direction.y() * t)
    }

    fn derivative(&self, _: f64) -> Vector {
        self.direction.to_vector()
    }

    fn length(&self) -> f64 {
        f64::INFINITY
    }
}
*/
