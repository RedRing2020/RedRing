//! Point - CAD統合層の点構造体
//!
//! geo_foundationのトレイトを実装したラッパー構造体

use crate::geometry3d::{Point3D as GeoPoint3D, Vector3D};
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point3D as Point3DTrait};
use geo_foundation::abstract_types::{ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]
pub struct Point {
    inner: GeoPoint3D,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::new(x, y, z),
        }
    }

    pub fn x(&self) -> f64 {
        self.inner.x()
    }
    pub fn y(&self) -> f64 {
        self.inner.y()
    }
    pub fn z(&self) -> f64 {
        self.inner.z()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.tolerant_eq_default(other)
    }
}

impl TolerantEq for Point {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.inner.tolerant_eq(&other.inner, context)
    }
}

impl PointTrait<3> for Point {
    type Scalar = f64;
    type Vector = Vector3D;

    fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn distance_to(&self, other: &Self) -> Self::Scalar {
        self.inner.distance_to(&other.inner)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        let translated = self.inner.translate(vector);
        Self { inner: translated }
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        self.inner.vector_to(&other.inner)
    }

    fn coords(&self) -> [Self::Scalar; 3] {
        self.inner.coords()
    }
}

impl Point3DTrait for Point {
    fn x(&self) -> Self::Scalar {
        self.inner.x()
    }

    fn y(&self) -> Self::Scalar {
        self.inner.y()
    }

    fn z(&self) -> Self::Scalar {
        self.inner.z()
    }

    fn from_components(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self {
        Self::new(x, y, z)
    }
}
