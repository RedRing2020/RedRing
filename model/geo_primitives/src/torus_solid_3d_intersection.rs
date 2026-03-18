//! TorusSolid3D の交差判定実装
//!
//! トーラス立体との交差判定を提供する。

use crate::{Point3D, TorusSolid3D};
use geo_contracts::BasicIntersection;
use geo_foundation::Scalar;

// ============================================================================
// BasicIntersection implementations for TorusSolid3D
// ============================================================================

/// TorusSolid3D と Point3D の交差判定
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for TorusSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // 点がトーラス立体内部または表面上にあるかチェック
        // distance_to_point を使用
        if self.distance_to_point(point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Note: 他の形状との交差判定は後続の実装で追加予定
// - TorusSolid3D vs LineSegment3D
// - TorusSolid3D vs Ray3D
// - TorusSolid3D vs Plane3D
// - TorusSolid3D vs Triangle3D
// - TorusSolid3D vs Circle3D
// - TorusSolid3D vs InfiniteLine3D
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torus_solid_point_intersection() {
        let torus = TorusSolid3D::standard(5.0, 2.0).unwrap();

        // 表面上の点
        let point_on_surface = Point3D::new(7.0, 0.0, 0.0);
        let result = torus.intersection_with(&point_on_surface, 1e-6);
        assert!(result.is_some());

        // 内部の点
        let point_inside = Point3D::new(4.0, 0.0, 0.0);
        let result = torus.intersection_with(&point_inside, 1e-6);
        assert!(result.is_some());

        // 外部の点
        let point_outside = Point3D::new(100.0, 100.0, 100.0);
        let result = torus.intersection_with(&point_outside, 1e-6);
        assert!(result.is_none());
    }
}
