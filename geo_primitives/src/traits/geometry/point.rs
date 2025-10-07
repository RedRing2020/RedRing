/// Point - 基本点構造体

use crate::geometry3d::Point3D as GeoPoint3D;

#[derive(Debug, Clone)]
pub struct Point {
    inner: GeoPoint3D,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { inner: GeoPoint3D::new(x, y, z) }
    }

    pub fn x(&self) -> f64 { self.inner.x() }
    pub fn y(&self) -> f64 { self.inner.y() }
    pub fn z(&self) -> f64 { self.inner.z() }
}
