use super::{point::Point, direction::Direction};
use crate::model::geometry::geom2d::{line::Line, kind::CurveKind2, curve::curve_trait::Curve2};
use crate::model::geometry::geom2d::intersect::{IntersectionResult, IntersectionKind, EPSILON};

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point,
    radius: f64,
    direction: Direction, // 回転方向（通常は正方向 = 反時計回り）
}

impl Curve2 for Circle {
    fn kind(&self) -> CurveKind2 {
        CurveKind2::Circle
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point, direction::Direction};

    #[test]
    fn test_evaluate() {
        let c = Circle::new(Point::new(0.0, 0.0), 5.0, Direction::new(0.0, 1.0));
        let p = c.evaluate(0.0);
        assert_eq!(p, Point::new(5.0, 0.0));

        let p90 = c.evaluate(std::f64::consts::FRAC_PI_2);
        assert!((p90.x).abs() < 1e-10);
        assert!((p90.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_tangent_and_normal() {
        let c = Circle::new(Point::new(0.0, 0.0), 1.0, Direction::new(0.0, 1.0));
        let t = c.tangent(0.0);
        assert_eq!(t, Direction::new(0.0, 1.0));

        let n = c.normal(0.0);
        assert_eq!(n, Direction::new(1.0, 0.0));
    }
}