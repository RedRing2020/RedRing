use crate::model::geometry::geom2d::{
    point::Point,
    direction::Direction,
    line::Line,
    ellipse::Ellipse,
    intersection_result::{IntersectionResult, IntersectionKind},
};
use crate::model::geometry_kind::CurveKind2D;
use crate::model::geometry_trait::curve2d::Curve2D;

use crate::model::analysis::{consts::EPSILON, sampling2d::sample_intersections};


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

    pub fn intersection_with_ellipse(&self, ellipse: &Ellipse, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |θ| ellipse.evaluate(θ),
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

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
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
}

impl Curve2D for Ray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Ray
    }

    fn evaluate(&self, t: f64) -> Point {
        self.origin.add(self.direction.x * t, self.direction.y * t)
    }

    fn derivative(&self, _: f64) -> Direction {
        self.direction.clone()
    }

    fn length(&self) -> f64 {
        f64::INFINITY
    }
}
