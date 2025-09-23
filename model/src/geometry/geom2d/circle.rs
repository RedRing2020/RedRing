use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Circle2D {
    center: Point2D,
    radius: f64,
    direction: Direction2D, // 回転方向（通常は正方向 = 反時計回り）
}

impl Circle2D {
    pub fn new(center: Point2D, radius: f64, direction: Direction2D) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
            direction,
        }
    }

    pub fn center(&self) -> &Point2D {
        &self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    /// θ ∈ [0, 2π) における点を評価
    pub fn evaluate(&self, theta: f64) -> Point2D {
        let dx = self.radius * theta.cos();
        let dy = self.radius * theta.sin();
        self.center.add(dx, dy)
    }

    /// θ ∈ [0, 2π) における接線方向（右手系）
    pub fn tangent(&self, theta: f64) -> Direction2D {
        Direction2D::new(-theta.sin(), theta.cos())
    }

    /// θ ∈ [0, 2π) における法線方向（中心から外向き）
    pub fn normal(&self, theta: f64) -> Direction2D {
        Direction2D::new(theta.cos(), theta.sin())
    }

    /// 線分との交差点を求める
    pub fn intersection_with_line(&self, line: &Line2D) -> IntersectionResult2D {
        let pts = line.intersection_with_circle(self);
        let kind = match pts.len() {
            0 => IntersectionKind2D::None,
            1 => IntersectionKind2D::Tangent,
            _ => IntersectionKind2D::Point,
        };
        IntersectionResult2D {
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
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D};

    #[test]
    fn test_evaluate() {
        let c = Circle2D::new(Point2D::new(0.0, 0.0), 5.0, Direction2D::new(0.0, 1.0));
        let p = c.evaluate(0.0);
        assert_eq!(p, Point2D::new(5.0, 0.0));

        let p90 = c.evaluate(std::f64::consts::FRAC_PI_2);
        assert!((p90.x).abs() < 1e-10);
        assert!((p90.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_tangent_and_normal() {
        let c = Circle2D::new(Point2D::new(0.0, 0.0), 1.0, Direction2D::new(0.0, 1.0));
        let t = c.tangent(0.0);
        assert_eq!(t, Direction2D::new(0.0, 1.0));

        let n = c.normal(0.0);
        assert_eq!(n, Direction2D::new(1.0, 0.0));
    }
}