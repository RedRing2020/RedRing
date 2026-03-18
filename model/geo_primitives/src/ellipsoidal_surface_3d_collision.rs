//! EllipsoidalSurface3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装

use crate::{EllipsoidalSurface3D, Point3D};
use geo_contracts::BasicCollision;
use geo_foundation::Scalar;

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// EllipsoidalSurface3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for EllipsoidalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // サーフェスは面なので、点との overlap は intersects と同じ
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.distance_to_surface(point)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::BasicCollision;

    #[test]
    fn test_ellipsoidal_surface_collision_with_point() {
        // 原点中心、半径 (2, 3, 4) の楕円体サーフェス
        let surface = EllipsoidalSurface3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // サーフェス上の点（X軸上）
        let point_on_surface = Point3D::new(2.0, 0.0, 0.0);
        assert!(surface.intersects(&point_on_surface, 1e-10));

        // サーフェス外の点
        let point_outside = Point3D::new(10.0, 0.0, 0.0);
        assert!(!surface.intersects(&point_outside, 1e-10));

        // サーフェス内の点
        let point_inside = Point3D::new(0.5, 0.5, 0.5);
        let distance = surface.distance_to(&point_inside);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_ellipsoidal_surface_distance() {
        let surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();

        // 原点（中心）からの距離は半径に等しい（球の場合）
        let origin = Point3D::origin();
        let distance = surface.distance_to(&origin);
        assert!((distance - 1.0).abs() < 1e-6);

        // X軸上の点
        let point_x = Point3D::new(2.0, 0.0, 0.0);
        let distance_x = surface.distance_to(&point_x);
        assert!((distance_x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ellipsoidal_surface_tolerance() {
        let surface = EllipsoidalSurface3D::new_at_origin(5.0, 3.0, 2.0).unwrap();

        // サーフェスに近い点
        let near_point = Point3D::new(5.0, 0.0, 0.1);
        assert!(surface.intersects(&near_point, 1.0));
        assert!(!surface.intersects(&near_point, 0.001));
    }
}
