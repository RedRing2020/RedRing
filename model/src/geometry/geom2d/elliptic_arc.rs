use super::{ellipse::Ellipse2D, point::Point2D};

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
}