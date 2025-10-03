/// 点プリミティブの定義
///
/// 2D/3D空間における基本的な点要素

use geo_core::{Point2D as GeoPoint2D, Point3D as GeoPoint3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 2D点プリミティブ
#[derive(Debug, Clone)]
pub struct Point2D {
    inner: GeoPoint2D,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            inner: GeoPoint2D::from_f64(x, y),
        }
    }

    pub fn from_geo_core(point: GeoPoint2D) -> Self {
        Self { inner: point }
    }

    pub fn x(&self) -> f64 {
        self.inner.x().value()
    }

    pub fn y(&self) -> f64 {
        self.inner.y().value()
    }

    pub fn as_geo_core(&self) -> &GeoPoint2D {
        &self.inner
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }
}

impl GeometricPrimitive for Point2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Point
    }

    fn bounding_box(&self) -> BoundingBox {
        let point_3d = geo_core::Point3D::new(
            *self.inner.x(),
            *self.inner.y(),
            geo_core::Scalar::new(0.0),
        );
        BoundingBox::new(point_3d.clone(), point_3d)
    }

    fn measure(&self) -> Option<f64> {
        None // 点に面積はない
    }
}

/// 3D点プリミティブ
#[derive(Debug, Clone)]
pub struct Point3D {
    inner: GeoPoint3D,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::from_f64(x, y, z),
        }
    }

    pub fn from_geo_core(point: GeoPoint3D) -> Self {
        Self { inner: point }
    }

    pub fn x(&self) -> f64 {
        self.inner.x().value()
    }

    pub fn y(&self) -> f64 {
        self.inner.y().value()
    }

    pub fn z(&self) -> f64 {
        self.inner.z().value()
    }

    pub fn as_geo_core(&self) -> &GeoPoint3D {
        &self.inner
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }
}

impl GeometricPrimitive for Point3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Point
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.inner.clone(), self.inner.clone())
    }

    fn measure(&self) -> Option<f64> {
        None // 点に体積はない
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_creation() {
        let point = Point2D::new(1.0, 2.0);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.primitive_kind(), PrimitiveKind::Point);
    }

    #[test]
    fn test_point3d_creation() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.z(), 3.0);
        assert_eq!(point.primitive_kind(), PrimitiveKind::Point);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_bounding_box() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let bbox = point.bounding_box();
        assert_eq!(bbox.min.x().value(), 1.0);
        assert_eq!(bbox.max.z().value(), 3.0);
    }
}
