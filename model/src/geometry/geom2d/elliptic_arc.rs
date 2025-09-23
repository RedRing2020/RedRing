use super::{ellipse::Ellipse2D, point::Point2D};

use crate::model::analysis::sampling2d::sample_intersections;
use crate::model::analysis::consts::EPSILON;
use crate::model::geometry::geom2d::{point::Point2D, line::Line2D};

#[derive(Debug, Clone, PartialEq)]
pub struct EllipticArc2D {
    ellipse: Ellipse2D,
    start_angle: f64, // ラジアン [0, 2π)
    end_angle: f64,   // ラジアン [0, 2π)
}

impl EllipticArc2D {
    pub fn new(ellipse: Ellipse2D, start_angle: f64, end_angle: f64) -> Self {
        Self { ellipse, start_angle, end_angle }
    }

    pub fn sweep_angle(&self) -> f64 {
        let raw = self.end_angle - self.start_angle;
        if raw >= 0.0 { raw } else { raw + std::f64::consts::TAU }
    }

    pub fn evaluate(&self, t: f64) -> Point2D {
        let angle = self.start_angle + t * self.sweep_angle();
        self.ellipse.evaluate(angle)
    }

    pub fn start_point(&self) -> Point2D {
        self.evaluate(0.0)
    }

    pub fn end_point(&self) -> Point2D {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point2D {
        self.evaluate(0.5)
    }

    pub fn intersection_with_line(&self, line: &Line2D) -> Vec<Point2D> {
        let candidates = sample_intersections(
            |theta| self.ellipse.evaluate(theta),
            line,
            360,
            EPSILON,
        );

        candidates
            .into_iter()
            .filter(|pt| {
                let theta = self.ellipse.angle_of(pt);
                self.contains_angle(theta)
            })
            .collect()
    }

    /// 楕円弧の角度範囲に含まれるか（方向付き）
    pub fn contains_angle(&self, theta: f64) -> bool {
        let start = self.start_angle;
        let end = self.end_angle;

        let sweep = if end >= start {
            end - start
        } else {
            end + std::f64::consts::TAU - start
        };

        let rel = if theta >= start {
            theta - start
        } else {
            theta + std::f64::consts::TAU - start
        };

        rel >= -EPSILON && rel <= sweep + EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D, ellipse::Ellipse2D};

    #[test]
    fn test_evaluate_arc() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let arc = EllipticArc2D::new(ellipse, 0.0, std::f64::consts::FRAC_PI_2);

        let start = arc.start_point();
        let end = arc.end_point();
        let mid = arc.midpoint();

        assert_eq!(start, Point2D::new(5.0, 0.0));
        assert!((end.x).abs() < 1e-10);
        assert!((end.y - 3.0).abs() < 1e-10);
        assert!((mid.x - 3.5355).abs() < 1e-3);
        assert!((mid.y - 2.1213).abs() < 1e-3);
    }

    #[test]
    fn test_elliptic_arc_line_intersection_inside_range() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let arc = EllipticArc2D::new(ellipse, 0.0, std::f64::consts::FRAC_PI_2); // 0〜90度
        let line = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0));
        let pts = arc.intersection_with_line(&line);
        assert_eq!(pts.len(), 1);
    }

    #[test]
    fn test_elliptic_arc_line_intersection_outside_range() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let arc = EllipticArc2D::new(ellipse, std::f64::consts::PI, std::f64::consts::PI * 1.5); // 180〜270度
        let line = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0));
        let pts = arc.intersection_with_line(&line);
        assert_eq!(pts.len(), 0);
    }
}