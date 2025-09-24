use super::point::Point;
use super::vector::Vector;
use super::surface::Surface;
use super::kind::SurfaceKind;

/// A sphere surface defined by center and radius
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Surface for Sphere {
    fn evaluate(&self, u: f64, v: f64) -> Point {
        let x = self.radius * u.cos() * v.sin();
        let y = self.radius * u.sin() * v.sin();
        let z = self.radius * v.cos();
        self.center + Vector::new(x, y, z)
    }

    fn derivative_u(&self, u: f64, v: f64) -> Vector {
        let dx = -self.radius * u.sin() * v.sin();
        let dy = self.radius * u.cos() * v.sin();
        let dz = 0.0;
        Vector::new(dx, dy, dz)
    }

    fn derivative_v(&self, u: f64, v: f64) -> Vector {
        let dx = self.radius * u.cos() * v.cos();
        let dy = self.radius * u.sin() * v.cos();
        let dz = -self.radius * v.sin();
        Vector::new(dx, dy, dz)
    }

    fn parameter_range_u(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn parameter_range_v(&self) -> (f64, f64) {
        (0.0, std::f64::consts::PI)
    }

    fn is_closed_u(&self) -> bool {
        true
    }

    fn is_closed_v(&self) -> bool {
        false
    }

    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Sphere
    }
}