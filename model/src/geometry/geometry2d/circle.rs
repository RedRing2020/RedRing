use std::any::Any;

use crate::model::geometry_trait::curve2d::Curve2D;
use crate::model::geometry_common::{IntersectionResult, IntersectionKind};
use crate::model::geometry_kind::CurveKind2D;

use crate::model::geometry::geometry2d::{
    point::Point,
    direction::Direction,
    line::Line,
    arc::Arc,
    ellipse::Ellipse,
};

use crate::model::analysis::consts::EPSILON;
use crate::model::analysis::sampling2d::sample_intersections;

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point,
    radius: f64,
    direction: Direction, // 回転方向（通常は正方向 = 反時計回り）
}

impl Circle {
    pub fn new(center: Point, radius: f64, direction: Direction) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
            direction,
        }
    }

    pub fn center(&self) -> &Point {
        &self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn direction(&self) -> &Direction {
        &self.direction
    }

    /// θ ∈ [0, 2π) における点を評価
    pub fn evaluate(&self, theta: f64) -> Point {
        let dx = self.radius * theta.cos();
        let dy = self.radius * theta.sin();
        self.center.add(dx, dy)
    }

    /// θ ∈ [0, 2π) における接線方向（右手系）
    pub fn tangent(&self, theta: f64) -> Direction {
        Direction::new(-theta.sin(), theta.cos())
    }

    /// θ ∈ [0, 2π) における法線方向（中心から外向き）
    pub fn normal(&self, theta: f64) -> Direction {
        Direction::new(theta.cos(), theta.sin())
    }

    /// 無限直線との交差点を求める
    pub fn intersection_with_infinite_line(&self, line: &InfiniteLine, epsilon: f64) -> IntersectionResult {
        let dx = line.origin().x - self.center.x;
        let dy = line.origin().y - self.center.y;

        let normal = line.direction().right_hand_normal();
        let dist = dx * normal.x + dy * normal.y;

        if dist.abs() > self.radius + epsilon {
            return IntersectionResult::none(epsilon);
        }

        let d = dist.clamp(-self.radius, self.radius);
        let offset = Direction::new(normal.x * d, normal.y * d);
        let foot = self.center.add(offset.x, offset.y);

        let h = (self.radius.powi(2) - d.powi(2)).sqrt();
        let dir = line.direction();
        let p1 = foot.add(dir.x * h, dir.y * h);
        let p2 = foot.add(-dir.x * h, -dir.y * h);

        let kind = if h < epsilon {
            IntersectionKind::Tangent
        } else {
            IntersectionKind::Point
        };

        IntersectionResult {
            kind,
            points: if h < epsilon { vec![foot] } else { vec![p1, p2] },
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    /// 線分との交差点を求める
    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult {
        let pts = line.intersection_with_circle(self);
        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };
        IntersectionResult {
            kind,
            points: pts,
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn intersection_with_arc(&self, arc: &Arc, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| arc.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let θ = arc.angle_of(&pt);
                let t = (θ - arc.start_angle).rem_euclid(std::f64::consts::TAU) / arc.sweep_angle();
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
                let θ = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(θ);
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

impl Curve2 for Circle {
    fn kind(&self) -> CurveKind2 {
        CurveKind2::Circle
    }
}
