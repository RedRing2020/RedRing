/// Vector - 基本ベクトル構造体

use crate::geometry3d::Vector3D as GeoVector3D;

#[derive(Debug, Clone)]
pub struct Vector {
    inner: GeoVector3D,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { inner: GeoVector3D::new(x, y, z) }
    }

    pub fn x(&self) -> f64 { self.inner.x() }
    pub fn y(&self) -> f64 { self.inner.y() }
    pub fn z(&self) -> f64 { self.inner.z() }
}
