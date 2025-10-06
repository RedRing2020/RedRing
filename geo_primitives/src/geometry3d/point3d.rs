/// f64ベース3D点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D点への投影（Z座標を破棄）
    pub fn to_point2d(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.x, self.y)
    }

    /// XY平面上での距離計算
    pub fn xy_distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Point3D {
    fn default() -> Self {
        Self::origin()
    }
}

// 算術演算の実装
impl std::ops::Add<crate::geometry3d::Vector3D> for Point3D {
    type Output = Point3D;

    fn add(self, vec: crate::geometry3d::Vector3D) -> Self::Output {
        Point3D::new(
            self.x + vec.x(),
            self.y + vec.y(),
            self.z + vec.z()
        )
    }
}

impl std::ops::Sub<Point3D> for Point3D {
    type Output = crate::geometry3d::Vector3D;

    fn sub(self, other: Point3D) -> Self::Output {
        crate::geometry3d::Vector3D::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z
        )
    }
}

impl std::fmt::Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point3D({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3d_creation() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_point3d_origin() {
        let origin = Point3D::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::origin();
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        let distance = p1.distance_to(&p2);
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point3d_to_point2d() {
        let p3d = Point3D::new(1.0, 2.0, 3.0);
        let p2d = p3d.to_point2d();
        assert_eq!(p2d.x(), 1.0);
        assert_eq!(p2d.y(), 2.0);
    }
}
