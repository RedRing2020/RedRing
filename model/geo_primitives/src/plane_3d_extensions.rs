//! Plane3D 拡張機能実装
//!
//! 平面の基本的な幾何操作と計算機能

use crate::{Plane3D, Point3D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Basic Geometric Operations
// ============================================================================

impl<T: Scalar> Plane3D<T> {
    /// 平面による点の鏡像反転
    pub fn mirror_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let _distance = self.distance_to_point(*point);
        let normal = self.normal();

        // 符号付き距離を計算（平面の法線方向が正）
        let to_point = Vector3D::new(
            point.x() - self.point().x(),
            point.y() - self.point().y(),
            point.z() - self.point().z(),
        );

        let signed_distance = to_point.dot(&normal);
        let mirror_vector = normal * (signed_distance * (T::ONE + T::ONE));

        Point3D::new(
            point.x() - mirror_vector.x(),
            point.y() - mirror_vector.y(),
            point.z() - mirror_vector.z(),
        )
    }

    /// 平面によるベクトルの鏡像反転
    pub fn mirror_vector(&self, vector: &Vector3D<T>) -> Vector3D<T> {
        let normal = self.normal();
        let dot_product = vector.dot(&normal);
        let reflection = normal * (dot_product * (T::ONE + T::ONE));

        Vector3D::new(
            vector.x() - reflection.x(),
            vector.y() - reflection.y(),
            vector.z() - reflection.z(),
        )
    }

    /// 平面上への点の射影（参照版）
    pub fn project_point_ref(&self, point: &Point3D<T>) -> Point3D<T> {
        let distance = self.distance_to_point(*point);
        let projection_vector = self.normal() * distance;

        Point3D::new(
            point.x() - projection_vector.x(),
            point.y() - projection_vector.y(),
            point.z() - projection_vector.z(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_point() {
        let plane_point = Point3D::new(0.0, 0.0, 0.0);
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(plane_point, plane_normal).unwrap();

        let test_point = Point3D::new(1.0, 2.0, 3.0);
        let mirrored = plane.mirror_point(&test_point);

        let expected = Point3D::new(1.0, 2.0, -3.0);
        assert!((mirrored.x() - expected.x()).abs() < 1e-10);
        assert!((mirrored.y() - expected.y()).abs() < 1e-10);
        assert!((mirrored.z() - expected.z()).abs() < 1e-10);
    }

    #[test]
    fn test_mirror_vector() {
        let plane_point = Point3D::new(0.0, 0.0, 0.0);
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(plane_point, plane_normal).unwrap();

        let test_vector = Vector3D::new(1.0, 2.0, 3.0);
        let mirrored = plane.mirror_vector(&test_vector);

        let expected = Vector3D::new(1.0, 2.0, -3.0);
        assert!((mirrored.x() - expected.x()).abs() < 1e-10);
        assert!((mirrored.y() - expected.y()).abs() < 1e-10);
        assert!((mirrored.z() - expected.z()).abs() < 1e-10);
    }

    #[test]
    fn test_project_point() {
        let plane_point = Point3D::new(0.0, 0.0, 0.0);
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(plane_point, plane_normal).unwrap();

        let test_point = Point3D::new(1.0, 2.0, 5.0);
        let projected = plane.project_point_ref(&test_point);

        let expected = Point3D::new(1.0, 2.0, 0.0);
        assert!((projected.x() - expected.x()).abs() < 1e-10);
        assert!((projected.y() - expected.y()).abs() < 1e-10);
        assert!((projected.z() - expected.z()).abs() < 1e-10);
    }
}
