use crate::geometry::geometry2d::{
    point::Point,
    direction::Direction,
};
use crate::geometry_common::{IntersectionResult, IntersectionKind};
use crate::analysis::consts::EPSILON;

#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine {
    origin: Point,
    direction: Direction,
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
        let normal = self.direction.right_hand_normal();
        let dot = v[0] * normal.x + v[1] * normal.y;
        dot.abs()
    }

    pub fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        let v = [point.x - self.origin.x, point.y - self.origin.y];
        let cross = v[0] * self.direction.y - v[1] * self.direction.x;
        cross.abs() < epsilon
    }

    pub fn intersection_with_infinite_line(&self, other: &InfiniteLine, epsilon: f64) -> IntersectionResult<Point> {
        let dx = other.origin.x - self.origin.x;
        let dy = other.origin.y - self.origin.y;

        let det = self.direction.x * other.direction.y - self.direction.y * other.direction.x;

        if det.abs() < epsilon {
            if self.contains_point(&other.origin, epsilon) {
                return IntersectionResult::overlap(epsilon);
            } else {
                return IntersectionResult::none(epsilon);
            }
        }

        let t = (dx * other.direction.y - dy * other.direction.x) / det;
        let pt = self.evaluate(t);

        IntersectionResult::point(pt, t, epsilon)
    }

    pub fn intersection_with_line(&self, line: &Line, epsilon: f64) -> IntersectionResult<Point> {
        let result = self.intersection_with_infinite_line(&line.to_infinite(), epsilon);
        if let Some(pt) = result.points.first() {
            if line.contains_point(pt, epsilon) {
                result
            } else {
                IntersectionResult::none(epsilon)
            }
        } else {
            result
        }
    }
}

impl Curve2D for InfiniteLine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::InfiniteLine
    }

    fn evaluate(&self, t: f64) -> Point {
        self.evaluate(t)
    }

    fn derivative(&self, _: f64) -> Direction {
        self.direction.clone()
    }

    fn length(&self) -> f64 {
        f64::INFINITY // 無限長
    }
}
