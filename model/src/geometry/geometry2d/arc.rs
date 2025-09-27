use std::any::Any;

use crate::model::geometry_trait::{Curve2D, Intersect2D};
use crate::model::geometry_common::{IntersectionResult, IntersectionKind};
use crate::model::geometry_kind::CurveKind2D;

use crate::model::geometry::geometry2d::{
    point::Point,
    direction::Direction,
    infinite_line::InfiniteLine,
    ray::Ray,
    line::Line,
    circle::Circle,
    arc::Arc,
    ellipse::Ellipse,
    ellipse_arc::EllipseArc,
};

use crate::model::analysis::consts::EPSILON;
use crate::model::analysis::sampling2d::sample_intersections;
use crate::model::analysis::numeric::newton_inverse;

#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    circle: Circle,
    start_angle: f64,
    end_angle: f64,
}

impl Arc {
    pub fn new(circle: Circle, start_angle: f64, end_angle: f64) -> Self {
        Self {
            circle,
            start_angle: start_angle % std::f64::consts::TAU,
            end_angle: end_angle % std::f64::consts::TAU,
        }
    }

    pub fn center(&self) -> &Point {
        self.circle.center()
    }

    pub fn radius(&self) -> f64 {
        self.circle.radius()
    }

    pub fn direction(&self) -> &Direction {
        self.circle.direction()
    }

    pub fn sweep_angle(&self) -> f64 {
        let raw = (self.end_angle - self.start_angle + std::f64::consts::TAU) % std::f64::consts::TAU;
        if self.direction().is_ccw() {
            raw
        } else {
            std::f64::consts::TAU - raw
        }
    }

    pub fn start_point(&self) -> Point {
        self.evaluate(0.0)
    }

    pub fn end_point(&self) -> Point {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point {
        self.evaluate(0.5)
    }

    pub fn tangent(&self, t: f64) -> Direction {
        let angle = self.start_angle + t * self.sweep_angle();
        Direction::new(-angle.sin(), angle.cos())
    }

    pub fn normal(&self, t: f64) -> Direction {
        let angle = self.start_angle + t * self.sweep_angle();
        Direction::new(angle.cos(), angle.sin())
    }

    pub fn contains_angle(&self, theta: f64, epsilon: f64) -> bool {
        let sweep = self.sweep_angle();
        let normalized = (theta - self.start_angle + std::f64::consts::TAU) % std::f64::consts::TAU;
        normalized <= sweep + epsilon
    }

    pub fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        if !self.circle.contains_point(point, epsilon) {
            return false;
        }
        let dx = point.x - self.center().x;
        let dy = point.y - self.center().y;
        let theta = dy.atan2(dx);
        self.contains_angle(theta, epsilon)
    }

    pub fn trim_to(&mut self, t_start: f64, t_end: f64) {
        let sweep = self.sweep_angle();
        let new_sweep = (t_end - t_start) * sweep;
        self.start_angle = (self.start_angle + t_start * sweep) % std::f64::consts::TAU;
        self.end_angle = (self.start_angle + new_sweep) % std::f64::consts::TAU;
    }

    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start_angle, &mut self.end_angle);
        self.circle.reverse_direction();
    }

    pub fn intersection_with_infinite_line(&self, line: &InfiniteLine, epsilon: f64) -> IntersectionResult<Point> {
        let result = self.circle.intersection_with_infinite_line(line, epsilon);

        let mut pts = vec![];
        let mut params = vec![];

        for pt in result.points {
            if self.contains_point(&pt, epsilon) {
                let dx = pt.x - self.center().x;
                let dy = pt.y - self.center().y;
                let theta = dy.atan2(dx);
                let t = ((theta - self.start_angle + std::f64::consts::TAU) % std::f64::consts::TAU) / self.sweep_angle();
                pts.push(pt);
                params.push(t);
            }
        }

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: params,
            tolerance_used: epsilon,
        }
    }

    pub fn intersection_with_ray(&self, ray: &Ray, epsilon: f64) -> IntersectionResult {
        let line = Line::new(ray.origin, ray.origin.add(&ray.direction.to_vector()));
        let candidates = line.intersection_with_circle(&self.circle, epsilon);
        let pts = candidates
            .into_iter()
            .filter(|pt| self.contains_point(pt, epsilon) && ray.is_forward(pt))
            .collect::<Vec<_>>();

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    pub fn intersection_with_line(&self, line: &Line, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = line.intersection_with_infinite_line(self.circle.to_infinite(), epsilon);

        let mut pts = vec![];
        let mut params = vec![];

        for pt in candidates.points {
            if self.contains_point(&pt, epsilon) {
                let dx = pt.x - self.center().x;
                let dy = pt.y - self.center().y;
                let theta = dy.atan2(dx);
                let t = ((theta - self.start_angle + std::f64::consts::TAU) % std::f64::consts::TAU) / self.sweep_angle();
                pts.push(pt);
                params.push(t);
            }
        }

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: params,
            tolerance_used: epsilon,
        }
    }

    pub fn intersection_with_circle(&self, circle: &Circle, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |θ| circle.evaluate(θ),
            &self.to_line(),
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let θ = self.angle_of(&pt);
                let t = (θ - self.start_angle).rem_euclid(std::f64::consts::TAU) / self.sweep_angle();
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

    pub fn intersection_with_arc(&self, other: &Arc, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| other.evaluate(t),
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

    pub fn intersection_with_ellipse(&self, ellipse: &Ellipse, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |θ| ellipse.evaluate(θ),
            &self.to_line(),
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let θ = self.angle_of(&pt);
                let t = (θ - self.start_angle).rem_euclid(std::f64::consts::TAU) / self.sweep_angle();
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

    pub fn intersection_with_ellipse_arc(&self, ea: &EllipseArc, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |theta| ea.ellipse.evaluate(theta),
            &self.to_line(),
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            let theta = ea.ellipse.angle_of(&pt);
            if ea.contains_angle(theta) && self.contains_point(&pt, epsilon) {
                let t = (theta - ea.start_angle).rem_euclid(std::f64::consts::TAU) / ea.sweep_angle();
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

impl Curve2D for Arc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Arc
    }

    fn evaluate(&self, t: f64) -> Point {
        let angle = self.start_angle + t * self.sweep_angle();
        self.circle.evaluate(angle % std::f64::consts::TAU)
    }

    fn derivative(&self, t: f64) -> Direction {
        self.tangent(t)
    }

    fn length(&self) -> f64 {
        self.sweep_angle() * self.radius()
    }
}
