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
}

impl Curve3 for Circle {
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

    fn parameter_range(&self) -> (f64, f64) {
        (0.0, 2.0 * std::f64::consts::PI)
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn kind(&self) -> CurveKind3 {
        CurveKind3::Circle
    }
}