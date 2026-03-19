//! TorusSurface3D の交差判定実装
//!
//! トーラス曲面との交差判定を提供する。

use crate::{Point3D, TorusSurface3D};
use geo_contracts::BasicIntersection;
use geo_contracts::Scalar;

// ============================================================================
// BasicIntersection implementations for TorusSurface3D
// ============================================================================

/// TorusSurface3D と Point3D の交差判定
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for TorusSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // 点がトーラス表面上にあるかチェック
        let distance = self.distance_to(*point);
        if distance <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Note: 他の形状との交差判定は後続の実装で追加予定
// - TorusSurface3D vs LineSegment3D
// - TorusSurface3D vs Ray3D
// - TorusSurface3D vs Plane3D
// - TorusSurface3D vs Triangle3D
// - TorusSurface3D vs Circle3D
// - TorusSurface3D vs InfiniteLine3D
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torus_surface_point_intersection() {
        let torus = TorusSurface3D::standard(5.0, 2.0).unwrap();

        // 表面上の点
        let point_on_surface = torus.point_at(0.0, 0.0);
        let result = torus.intersection_with(&point_on_surface, 1e-6);
        assert!(result.is_some());

        // 表面外の点
        let point_outside = Point3D::new(100.0, 100.0, 100.0);
        let result = torus.intersection_with(&point_outside, 1e-6);
        assert!(result.is_none());
    }
}
