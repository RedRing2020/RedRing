//! Intersect2D: 2次元曲線の交差判定トレイト
//!
//! Curve2D を実装する構造体が交差判定を提供するための抽象インターフェース。
//! 誤差判定は `analysis::consts` に準拠。
use crate::geometry_kind::CurveKind2D;
use crate::geometry_trait::Curve2D;
use crate::geometry_common::{IntersectionResult, IntersectionKind};

use crate::geometry::geometry2d::{
    point::Point,
    direction::Direction,
    infinite_line::InfiniteLine,
    ray::Ray,
    line::Line,
    circle::Circle,
    arc::Arc,
    ellipse::Ellipse,
    ellipse_arc::EllipseArc,
    nurbs::NurbsCurve,
};

use crate::analysis::consts::EPSILON;

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
        match other.kind() {
            CurveKind2D::InfiniteLine => {
                let infinite_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                self.intersection_with_infinite_line(infinite_line, epsilon);
            }
            // Lineに委譲
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                line.intersection_with_infinite_line(self, epsilon)
            }
            // Circleに委譲
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                circle.intersection_with_line(self)
            }
            // Arcに委譲
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                arc.intersection_with_infinite_line(self, epsilon);
            }
            // Ellipseに委譲
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                ellipse.intersection_with_infinite_line(self, epsilon);
            }
            // EllipseArcに委譲
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                ellipse_arc.intersection_with_infinite_line(self, epsilon);
            }

            _ => IntersectionResult::none(epsilon),
        }
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
        match other.kind() {
            // InfiniteLineに委譲
            CurveKind2D::InfiniteLine => {
                let infinit_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                infinit_line.intersection_with_ray(self, epsilon);
            }
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                self.intersection_with_ray(ray, epsilon)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line)
            }
            // Circleに委譲
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                circle.intersection_with_ray(self, epsilon);
            }
            // Arcに委譲
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                arc.intersection_with_ray(self, epsilon);
            }
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                self.intersection_with_ellipse(ellipse, epsilon)
            }
            // EllipseArcに委譲
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                ellipse_arc.intersection_with_ray(self);
            }

            _ => IntersectionResult::none(epsilon),
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
        match other.kind() {
            CurveKind2D::InfiniteLine => {
                let infinit_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                self.intersection_with_infinite_line(infinit_line, epsilon);
            }
            // Rayに委譲
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                ray.intersection_with_line(self)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line, epsilon)
            }
            // Circleに委譲
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                circle.intersection_with_line(self);
            }
            // Arcに委譲
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                arc.intersection_with_line(self, epsilon);
            }
            // Ellipseに委譲
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                ellipse.intersection_with_line(self)
            }
            // EllipseArcに委譲
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                ellipse_arc.intersection_with_line(self);
            }

            _ => IntersectionResult::none(epsilon),
        }
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
        match other.kind() {
            // InfiniteLineに委譲
            CurveKind2D::InfiniteLine => {
                let infinite_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                infinite_line.intersection_with_infinite_circle(self, epsilon);
            }
            // Rayに委譲
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                ray.intersection_with_infinite_circle(self, epsilon)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line)
            }
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                self.intersection_with_infinite_circle(circle, epsilon);
            }
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                self.intersection_with_infinite_circle(arc, epsilon);
            }
            // Ellipseに委譲
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                ellipse.intersection_with_infinite_circle(self, epsilon)
            }
            // EllipseArcに委譲
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                ellipse_arc.intersection_with_line(self);
            }

            _ => IntersectionResult::none(epsilon),
        }
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

impl Intersect2D for Arc {
    fn intersects_with(&self, other: &dyn Curve2D, epsilon: f64) -> bool {
        self.intersection_result(other, epsilon).kind != IntersectionKind::None
    }

    fn intersection_points(&self, other: &dyn Curve2D, epsilon: f64) -> Vec<Point> {
        self.intersection_result(other, epsilon).points
    }

    fn intersection_result(&self, other: &dyn Curve2D, epsilon: f64) -> IntersectionResult {
        match other.kind() {
            CurveKind2D::InfiniteLine => {
                let infinit_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                self.intersection_with_infinit_line(infinit_line, epsilon);
            }
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                self.intersection_with_ray(ray, epsilon)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line, epsilon)
            }
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                self.intersection_with_circle(circle, epsilon);
            }
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                self.intersection_with_arc(arc, epsilon);
            }
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                self.intersection_with_ellipse(ellipse, epsilon)
            }
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                self.intersection_with_ellipse_arc(ellipse_arc, epsilon);
            }

            _ => IntersectionResult::none(epsilon),
        }
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
        match other.kind() {
            // InfineteLineに委譲
            CurveKind2D::InfiniteLine => {
                let infinit_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                infinit_line.intersection_with_ellipse(self, epsilon);
            }
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                self.intersection_with_ray(ray, epsilon)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line)
            }
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                self.intersection_with_circle(circle, epsilon);
            }
            // Arcに委譲
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                arc.intersection_with_ellipse(self, epsilon);
            }
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                self.intersection_with_ellipse(ellipse, epsilon)
            }
            // EllipseArcに委譲
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                ellipse_arc.intersection_with_ellipse(self, epsilon);
            }

            _ => IntersectionResult::none(epsilon),
        }
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
        match other.kind() {
            CurveKind2D::InfiniteLine => {
                let infinite_line = other.as_any().downcast_ref::<InfiniteLine>().unwrap();
                self.intersection_with_infinit_line(infinite_line, epsilon);
            }
            CurveKind2D::Ray => {
                let ray = other.as_any().downcast_ref::<Ray>().unwrap();
                self.intersection_with_ray(ray)
            }
            CurveKind2D::Line => {
                let line = other.as_any().downcast_ref::<Line>().unwrap();
                self.intersection_with_line(line)
            }
            CurveKind2D::Circle => {
                let circle = other.as_any().downcast_ref::<Circle>().unwrap();
                self.intersection_with_circle(circle, epsilon);
            }
            CurveKind2D::Arc => {
                let arc = other.as_any().downcast_ref::<Arc>().unwrap();
                self.intersection_with_arc(arc, epsilon);
            }
            CurveKind2D::Ellipse => {
                let ellipse = other.as_any().downcast_ref::<Ellipse>().unwrap();
                self.intersection_with_ellipse(ellipse, epsilon)
            }
            CurveKind2D::EllipseArc => {
                let ellipse_arc = other.as_any().downcast_ref::<EllipseArc>().unwrap();
                self.intersection_with_ellipse_arc(ellipse_arc, epsilon);
            }

            _ => IntersectionResult::none(epsilon),
        }
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
