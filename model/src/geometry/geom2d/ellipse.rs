use crate::model::geometry::geom2d::{point::Point, direction::Direction};
use crate::model::analysis::{sampling2d::sample_intersections, consts::EPSILON};
use crate::model::geometry::geom2d::{line::Line};
use crate::model::geometry::geom2d::kind::CurveKind2;
use crate::model::geometry::geom2d::curve::curve_trait::Curve2;

#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse {
    center: Point,
    major_axis: Direction, // 長軸方向（正規化済み）
    major_radius: f64,
    minor_radius: f64,
}

impl Curve2 for Ellipse {
    fn kind(&self) -> CurveKind2 {
        CurveKind2::Ellipse
    }
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

    /// 線分との交差点を離散近似で求める
    pub fn intersection_with_line(&self, line: &Line) -> Vec<Point> {
        sample_intersections(|theta| self.evaluate(theta), line, 360, EPSILON)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point, direction::Direction};

    #[test]
    fn test_evaluate_major_axis() {
        let ellipse = Ellipse::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0), 5.0, 3.0);
        let p = ellipse.evaluate(0.0);
        assert_eq!(p, Point::new(5.0, 0.0));
    }

    #[test]
    fn test_evaluate_minor_axis() {
        let ellipse = Ellipse::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0), 5.0, 3.0);
        let p = ellipse.evaluate(std::f64::consts::FRAC_PI_2);
        assert_eq!(p, Point::new(0.0, 3.0));
    }

    #[test]
    fn test_tangent_and_normal() {
        let ellipse = Ellipse::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0), 5.0, 3.0);
        let t = ellipse.tangent(0.0);
        assert_eq!(t, Direction::new(0.0, 1.0));

        let n = ellipse.normal(0.0);
        assert_eq!(n, Direction::new(1.0, 0.0));
    }

    #[test]
    fn test_ellipse_line_intersection_two_points() {
        let ellipse = Ellipse::new(Point::new(0.0, 0.0), Direction::new(1.0, 0.0), 5.0, 3.0);
        let line = Line::new(Point::new(-6.0, 0.0), Point::new(6.0, 0.0));
        let pts = ellipse.intersection_with_line(&line);
        assert_eq!(pts.len(), 2);
    }
}