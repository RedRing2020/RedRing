use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse2D {
    center: Point2D,
    major_axis: Direction2D, // 長軸方向（正規化済み）
    major_radius: f64,
    minor_radius: f64,
}

impl Ellipse2D {
    pub fn new(center: Point2D, major_axis: Direction2D, major_radius: f64, minor_radius: f64) -> Self {
        Self {
            center,
            major_axis,
            major_radius: major_radius.max(0.0),
            minor_radius: minor_radius.max(0.0),
        }
    }

    /// θ ∈ [0, 2π) における点を評価
    pub fn evaluate(&self, theta: f64) -> Point2D {
        let cos = theta.cos();
        let sin = theta.sin();
        let dx = self.major_axis.x() * self.major_radius * cos
               - self.major_axis.y() * self.minor_radius * sin;
        let dy = self.major_axis.y() * self.major_radius * cos
               + self.major_axis.x() * self.minor_radius * sin;
        self.center.add(dx, dy)
    }

    /// 接線方向（θにおける微分ベクトル）
    pub fn tangent(&self, theta: f64) -> Direction2D {
        let dx = -self.major_axis.x() * self.major_radius * theta.sin()
               - self.major_axis.y() * self.minor_radius * theta.cos();
        let dy = -self.major_axis.y() * self.major_radius * theta.sin()
               + self.major_axis.x() * self.minor_radius * theta.cos();
        Direction2D::new(dx, dy)
    }

    /// 法線方向（中心から外向き）
    pub fn normal(&self, theta: f64) -> Direction2D {
        let p = self.evaluate(theta);
        Direction2D::new(p.x - self.center.x, p.y - self.center.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D};

    #[test]
    fn test_evaluate_major_axis() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let p = ellipse.evaluate(0.0);
        assert_eq!(p, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_evaluate_minor_axis() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let p = ellipse.evaluate(std::f64::consts::FRAC_PI_2);
        assert_eq!(p, Point2D::new(0.0, 3.0));
    }

    #[test]
    fn test_tangent_and_normal() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), Direction2D::new(1.0, 0.0), 5.0, 3.0);
        let t = ellipse.tangent(0.0);
        assert_eq!(t, Direction2D::new(0.0, 1.0));

        let n = ellipse.normal(0.0);
        assert_eq!(n, Direction2D::new(1.0, 0.0));
    }
}