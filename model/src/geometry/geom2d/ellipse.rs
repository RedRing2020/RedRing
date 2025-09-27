use crate::model::geometry::geom2d::{
    point::Point,
    direction::Direction,
    ray::Ray,
    line::Line,
    circle::Circle,
    intersection_result::{IntersectionResult, IntersectionKind},
};

use crate::model::geometry::geom2d::kind::CurveKind2D;
use crate::model::geometry::geom2d::curve::curve_trait::Curve2D;

use crate::model::analysis::consts::EPSILON;
use crate::model::analysis::sampling2d::sample_intersections;
use crate::model::analysis::numeric::newton_inverse;

#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse {
    center: Point,
    major_axis: Direction, // 長軸方向（正規化済み）
    major_radius: f64,
    minor_radius: f64,
}

impl Ellipse {
    pub fn new(center: Point, major_axis: Direction, major_radius: f64, minor_radius: f64) -> Self {
        Self {
            center,
            major_axis,
            major_radius: major_radius.max(0.0),
            minor_radius: minor_radius.max(0.0),
        }
    }

    /// θ ∈ [0, 2π) における点を評価
    pub fn evaluate(&self, theta: f64) -> Point {
        let cos = theta.cos();
        let sin = theta.sin();
        let dx = self.major_axis.x() * self.major_radius * cos
               - self.major_axis.y() * self.minor_radius * sin;
        let dy = self.major_axis.y() * self.major_radius * cos
               + self.major_axis.x() * self.minor_radius * sin;
        self.center.add(dx, dy)
    }

    /// 接線方向（θにおける微分ベクトル）
    pub fn tangent(&self, theta: f64) -> Direction {
        let dx = -self.major_axis.x() * self.major_radius * theta.sin()
               - self.major_axis.y() * self.minor_radius * theta.cos();
        let dy = -self.major_axis.y() * self.major_radius * theta.sin()
               + self.major_axis.x() * self.minor_radius * theta.cos();
        Direction::new(dx, dy)
    }

    /// 法線方向（中心から外向き）
    pub fn normal(&self, theta: f64) -> Direction {
        let p = self.evaluate(theta);
        Direction::new(p.x - self.center.x, p.y - self.center.y)
    }

    pub fn intersection_with_ray(&self, ray: &Ray, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| ray.evaluate(t),
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

    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult<Point> {
        let candidates = sample_intersections(|theta| self.evaluate(theta), line, 360, EPSILON);
        let mut result = IntersectionResult::none(EPSILON);

        for pt in candidates {
            let v = pt.sub(&self.center);
            let initial_theta = v.y.atan2(v.x);

            let f = |theta: f64| self.evaluate(theta).distance_to(&pt);
            let df = |theta: f64| self.tangent(theta).dot(&self.normal(theta));

            if let Some(theta) = newton_inverse(f, df, 0.0, initial_theta, 20, EPSILON) {
                let t = theta.rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
                result.kind = IntersectionKind::Point;
                result.points.push(pt);
                result.parameters.push(t);
            }
        }

        result
    }

    pub fn intersection_with_circle(&self, circle: &Circle, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |θ| circle.evaluate(θ),
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

    pub fn intersection_with_circle(&self, circle: &Circle, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |θ| circle.evaluate(θ),
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

impl Curve2D for Ellipse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Ellipse
    }

    fn evaluate(&self, t: f64) -> Point {
        let theta = t.rem_euclid(1.0) * std::f64::consts::TAU;
        self.evaluate(theta)
    }

    fn derivative(&self, t: f64) -> Direction {
        let theta = t.rem_euclid(1.0) * std::f64::consts::TAU;
        self.tangent(theta)
    }

    fn length(&self) -> f64 {
        // Ramanujan の近似式（語義整合された楕円周長）
        let a = self.major_radius;
        let b = self.minor_radius;
        let h = ((a - b).powi(2)) / ((a + b).powi(2));
        std::f64::consts::PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }
}
