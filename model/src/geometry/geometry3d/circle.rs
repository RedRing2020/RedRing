use crate::geometry::geometry3d::{point::Point, vector::Vector, direction::Direction};
use crate::geometry_kind::curve3d::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub normal: Direction,
}

impl Circle {
    pub fn new(center: Point, radius: f64, normal: Direction) -> Self {
        Self { center, radius, normal }
    }

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn is_closed(&self) -> bool {
        true
    }
}

impl Curve3D for Circle {
    fn evaluate(&self, t: f64) -> Point {
        let u = self.normal.orthonormal_basis();
        let x = self.radius * t.cos();
        let y = self.radius * t.sin();
        self.center + u.0 * x + u.1 * y
    }

    fn derivative(&self, t: f64) -> Vector {
        let u = self.normal.orthonormal_basis();
        let dx = -self.radius * t.sin();
        let dy = self.radius * t.cos();
        u.0 * dx + u.1 * dy
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Circle
    }
}
