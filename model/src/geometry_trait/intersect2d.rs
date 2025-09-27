//! Intersect2D: 2次元曲線の交差判定トレイト
//!
//! Curve2D を実装する構造体が交差判定を提供するための抽象インターフェース。
//! 誤差判定は `analysis::consts` に準拠。
use std::any::Any;

use crate::model::geometry_trait::Curve2D;
use crate::model::geometry_common::{IntersectionResult, IntersectionKind};

use crate::model::geometry::geom2d::{
    point::Point,
    direction::Direction,
    line::Line,
    ray::Ray,
    arc::Arc,
    circle::Circle,
    ellipse::Ellipse,
    ellipse_arc::EllipseArc,
    infinite_line::InfiniteLine,
};

use crate::model::analysis::consts::EPSILON;
use crate::model::analysis::numeric::newton_project;

pub trait Intersect2D {
    /// 他の曲線との交差判定（結果の有無）
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool;

    /// 他の曲線との交点（Point のみ）
    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point>;

    /// 他の曲線との詳細な交差結果（Kind, 点, パラメータ, 誤差）
    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult;

    /// 指定点が曲線上にあるか（誤差付き）
    fn contains_point(&self, point: &Point, epsilon: f64) -> bool;

    /// 指定点との距離
    fn distance_to_point(&self, point: &Point) -> f64;
}

impl Intersect2D for InfiniteLine {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(line) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_infinite_line(&line.to_infinite(), epsilon);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return circle.intersection_with_infinite_line(self, epsilon);
        }
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return arc.intersection_with_infinite_line(self, epsilon);
        }
        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        InfiniteLine::contains_point(self, point, epsilon)
    }

    fn distance_to_point(&self, point: &Point) -> f64 {
        InfiniteLine::distance_to_point(self, point)
    }
}

impl Intersect2D for Ray {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return arc.intersection_with_ray(self, epsilon);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return circle.intersection_with_ray(self, epsilon);
        }
        if let Some(ellipse) = other.as_any().downcast_ref::<Ellipse>() {
            return self.intersection_with_ellipse(ellipse, epsilon);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        self.contains_point(pt, epsilon)
    }

    fn distance_to_point(&self, pt: &Point) -> f64 {
        let initial = 0.5;
        if let Some(p) = newton_project(|t| self.evaluate(t), pt, initial, EPSILON) {
            p.distance_to(pt)
        } else {
            f64::INFINITY
        }
    }
}

impl Intersect2D for Line {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(line2) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_line(line2, epsilon);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return circle.intersection_with_line(self, epsilon);
        }
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return arc.intersection_with_line(self, epsilon);
        }
        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        let on_line = self.to_infinite().contains_point(point, epsilon);
        let d1 = self.start.distance_to(point);
        let d2 = self.end.distance_to(point);
        on_line && (d1 + d2 <= self.length + epsilon)
    }

    fn distance_to_point(&self, point: &Point) -> f64 {
        self.to_infinite().distance_to_point(point)
    }
}

impl Intersect2D for Circle {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return self.intersection_with_arc(arc, epsilon);
        }
        if let Some(ellipse) = other.as_any().downcast_ref::<Ellipse>() {
            return self.intersection_with_ellipse(ellipse, epsilon);
        }
        if let Some(earc) = other.as_any().downcast_ref::<EllipseArc>() {
            return self.intersection_with_ellipse_arc(earc, epsilon);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        let d = self.center.distance_to(pt);
        (d - self.radius).abs() < epsilon
    }

    fn distance_to_point(&self, pt: &Point) -> f64 {
        let d = self.center.distance_to(pt);
        (d - self.radius).abs()
    }
}

use crate::model::geometry_trait::intersect2d::Intersect2D;
use crate::model::geometry_common::{IntersectionResult, IntersectionKind};
use crate::model::geometry::geom2d::{
    point::Point,
    line::Line,
    arc::Arc,
    ellipse_arc::EllipseArc,
    circle::Circle,
    ray::Ray,
};

impl Intersect2D for Arc {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(infinit_line) = other.as_any().downcast_ref::<InfiniteLine>() {
            return self.intersection_with_infinite_line(infinit_line, epsilon);
        }
        if let Some(ray) = other.as_any().downcast_ref::<Ray>() {
            return self.intersection_with_ray(ray, epsilon);
        }
        if let Some(line) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_line(line, epsilon);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return self.intersection_with_circle(circle, epsilon);
        }
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return self.intersection_with_arc(arc, epsilon);
        }
        if let Some(ellipse) = other.as_any().downcast_ref::<Ellipse>() {
            return self.intersection_with_ellipse(ellipse, epsilon);
        }
        if let Some(earc) = other.as_any().downcast_ref::<EllipseArc>() {
            return self.intersection_with_ellipse_arc(earc, epsilon);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, point: &Point, epsilon: f64) -> bool {
        self.contains_point(point, epsilon)
    }

    fn distance_to_point(&self, point: &Point) -> f64 {
        let initial = self.parameter_hint(point);
        if let Some(p) = newton_project(|t| self.evaluate(t), point, initial, EPSILON) {
            p.distance_to(point)
        } else {
            f64::INFINITY
        }
    }
}

impl Intersect2D for Ellipse {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(line) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_line(line);
        }
        if let Some(ray) = other.as_any().downcast_ref::<Ray>() {
            return ray.intersection_with_ellipse(self, epsilon);
        }
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return arc.intersection_with_ellipse(self, epsilon);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return self.intersection_with_circle(circle, epsilon);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        self.contains_point(pt, epsilon)
    }

    fn distance_to_point(&self, pt: &Point) -> f64 {
        let initial = Direction::from_points(&self.center, pt).angle();
        if let Some(p) = newton_project(|θ| self.evaluate(θ), pt, initial, EPSILON) {
            p.distance_to(pt)
        } else {
            self.center.distance_to(pt)
        }
    }
}

impl Intersect2D for EllipseArc {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult<Point> {
        if let Some(infinit_line) = other.as_any().downcast_ref::<InfiniteLine>() {
            return self.intersection_with_infinite_line(infinit_line, epsilon);
        }
        if let Some(ray) = other.as_any().downcast_ref::<Ray>() {
            return self.intersection_with_ray(ray);
        }
        if let Some(line) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_line(line);
        }
        if let Some(circle) = other.as_any().downcast_ref::<Circle>() {
            return self.intersection_with_circle(circle, epsilon);
        }
        if let Some(arc) = other.as_any().downcast_ref::<Arc>() {
            return arc.intersection_with_ellipse_arc(self, epsilon);
        }
        if let Some(ellipse) = other.as_any().downcast_ref::<Ellipse>() {
            return self.intersection_with_ellipse(ellipse, epsilon);
        }
        if let Some(other_arc) = other.as_any().downcast_ref::<EllipseArc>() {
            return self.intersection_with_ellipse_arc(other_arc, epsilon);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        let theta = self.ellipse.angle_of(pt);
        if !self.contains_angle(theta) {
            return false;
        }
        self.ellipse.evaluate(theta).distance_to(pt) < epsilon
    }

    fn distance_to_point(&self, pt: &Point) -> f64 {
        let initial = Direction::from_points(&self.ellipse.center(), pt).angle();
        if let Some(p) = newton_project(|θ| self.ellipse.evaluate(θ), pt, initial, EPSILON) {
            let theta = self.ellipse.angle_of(&p);
            if self.contains_angle(theta) {
                return p.distance_to(pt);
            }
        }
        f64::INFINITY
    }
}

impl Intersect2D for NurbsCurve {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        if let Some(line) = other.as_any().downcast_ref::<Line>() {
            return self.intersection_with_line(line);
        }

        IntersectionResult::none(epsilon)
    }

    fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        let initial = 0.5 * (self.domain.0 + self.domain.1);
        if let Some(u) = newton_project(|u| self.evaluate(u), pt, initial, epsilon) {
            self.evaluate(u).distance_to(pt) < epsilon
        } else {
            false
        }
    }

    fn distance_to_point(&self, pt: &Point) -> f64 {
        let initial = 0.5 * (self.domain.0 + self.domain.1);
        if let Some(p) = newton_project(|u| self.evaluate(u), pt, initial, EPSILON) {
            p.distance_to(pt)
        } else {
            f64::INFINITY
        }
    }
}
