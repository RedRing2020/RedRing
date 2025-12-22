//! EllipsoidalSurface3D 交差計算実装
//!
//! BasicIntersection トレイトの実装

use crate::{EllipsoidalSurface3D, Point3D};
use geo_foundation::{extensions::BasicIntersection, Scalar};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// EllipsoidalSurface3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for EllipsoidalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // 点がサーフェス上にある場合、その点を返す
        if self.distance_to_surface(point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::extensions::BasicIntersection;

    #[test]
    fn test_ellipsoidal_surface_intersection_with_point() {
        let surface = EllipsoidalSurface3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // サーフェス上の点
        let point_on_surface = Point3D::new(2.0, 0.0, 0.0);
        let intersection = surface.intersection_with(&point_on_surface, 1e-6);
        assert!(intersection.is_some());
        assert_eq!(intersection.unwrap(), point_on_surface);

        // サーフェス外の点
        let point_outside = Point3D::new(10.0, 0.0, 0.0);
        let intersection_outside = surface.intersection_with(&point_outside, 1e-6);
        assert!(intersection_outside.is_none());
    }

    #[test]
    fn test_ellipsoidal_surface_intersection_tolerance() {
        let surface = EllipsoidalSurface3D::new_at_origin(1.0, 1.0, 1.0).unwrap();

        // サーフェスに近い点（許容誤差内）
        let near_point = Point3D::new(1.0, 0.0, 0.05);

        // 実際の距離を確認
        let actual_distance = surface.distance_to_surface(&near_point);

        // 大きな許容誤差では交差あり
        assert!(surface.intersection_with(&near_point, 0.1).is_some());

        // 小さな許容誤差（実際の距離より小さい）では交差なし
        assert!(surface
            .intersection_with(&near_point, actual_distance / 2.0)
            .is_none());
    }
}
