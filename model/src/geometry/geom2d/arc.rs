use super::{point::Point2D, direction::Direction2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Arc2D {
    center: Point2D,
    radius: f64,
    start_angle: f64, // ラジアン [0, 2π)
    end_angle: f64,   // ラジアン [0, 2π)
    direction: Direction2D, // 回転方向（通常は反時計回り）
}

impl Arc2D {
    pub fn new(center: Point2D, radius: f64, start_angle: f64, end_angle: f64, direction: Direction2D) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
            start_angle,
            end_angle,
            direction,
        }
    }

    /// θ ∈ [0, 1] における補間点（0=start, 1=end）
    pub fn evaluate(&self, t: f64) -> Point2D {
        let angle = self.start_angle + t * self.sweep_angle();
        let dx = self.radius * angle.cos();
        let dy = self.radius * angle.sin();
        self.center.add(dx, dy)
    }

    /// 弧の角度（方向に応じて正規化）
    pub fn sweep_angle(&self) -> f64 {
        let raw = self.end_angle - self.start_angle;
        if raw >= 0.0 {
            raw
        } else {
            raw + std::f64::consts::TAU // 2π
        }
    }

    /// 始点
    pub fn start_point(&self) -> Point2D {
        self.evaluate(0.0)
    }

    /// 終点
    pub fn end_point(&self) -> Point2D {
        self.evaluate(1.0)
    }

    /// 中点
    pub fn midpoint(&self) -> Point2D {
        self.evaluate(0.5)
    }

    pub fn tangent(&self, t: f64) -> Direction2D {
        let angle = self.start_angle + t * self.sweep_angle();
        Direction2D::new(-angle.sin(), angle.cos()) // 右手系
    }

    pub fn normal(&self, t: f64) -> Direction2D {
        let angle = self.start_angle + t * self.sweep_angle();
        Direction2D::new(angle.cos(), angle.sin())
    }

    pub fn contains_angle(&self, theta: f64, epsilon: f64) -> bool {
        let sweep = self.sweep_angle();
        let normalized = (theta - self.start_angle + std::f64::consts::TAU) % std::f64::consts::TAU;
        normalized <= sweep + epsilon
    }

    pub fn contains_point(&self, point: &Point2D, epsilon: f64) -> bool {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
        if (dist - self.radius).abs() > epsilon {
            return false;
        }
        let angle = dy.atan2(dx);
        self.contains_angle(angle, epsilon)
    }

    /// t_start, t_end ∈ [0, 1] の範囲で弧をトリム
    pub fn trim_to(&mut self, t_start: f64, t_end: f64) {
        let sweep = self.sweep_angle();
        self.start_angle += t_start * sweep;
        self.end_angle = self.start_angle + (t_end - t_start) * sweep;
    }

    /// 弧の向きを反転
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start_angle, &mut self.end_angle);
        self.direction = self.direction.normal(); // 90度回転で反転
    }

    /// 円に変換
    pub fn to_circle(&self) -> Circle2D {
        Circle2D::new(self.center.clone(), self.radius, self.direction.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D};

    #[test]
    fn test_evaluate_arc() {
        let arc = Arc2D::new(Point2D::new(0.0, 0.0), 5.0, 0.0, std::f64::consts::FRAC_PI_2, Direction2D::new(0.0, 1.0));
        let start = arc.start_point();
        let end = arc.end_point();
        let mid = arc.midpoint();

        assert_eq!(start, Point2D::new(5.0, 0.0));
        assert!((end.x).abs() < 1e-10);
        assert!((end.y - 5.0).abs() < 1e-10);
        assert!((mid.x - 3.5355).abs() < 1e-3);
        assert!((mid.y - 3.5355).abs() < 1e-3);
    }

    #[test]
    fn test_contains_point_on_arc() {
        let arc = Arc2D::new(Point2D::new(0.0, 0.0), 5.0, 0.0, std::f64::consts::FRAC_PI_2, Direction2D::new(0.0, 1.0));
        let p = Point2D::new(0.0, 5.0);
        assert!(arc.contains_point(&p, 1e-10));
    }

    #[test]
    fn test_trim_and_reverse() {
        let mut arc = Arc2D::new(Point2D::new(0.0, 0.0), 5.0, 0.0, std::f64::consts::PI, Direction2D::new(0.0, 1.0));
        arc.trim_to(0.25, 0.75);
        let start = arc.start_point();
        let end = arc.end_point();
        assert!((start.x - 3.5355).abs() < 1e-3);
        assert!((end.x + 3.5355).abs() < 1e-3);

        arc.reverse();
        assert!((arc.start_point().x + 3.5355).abs() < 1e-3);
    }
}