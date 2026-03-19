//! TorusSurface3D の衝突判定実装
//!
//! トーラス曲面との衝突判定を提供する。
//! トーラス曲面は表面のみを持ち、内部は考慮しない。

use crate::{Point3D, TorusSurface3D};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// BasicCollision implementations for TorusSurface3D
// ============================================================================

/// TorusSurface3D と Point3D の衝突判定
impl<T: Scalar> BasicCollision<T, Point3D<T>> for TorusSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(*point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // Surface は厚みを持たないため、intersects と同じ
        self.distance_to(*point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // トーラス曲面までの距離を計算してから extensions の distance_to を呼ぶ
        // extensions の distance_to を直接呼ぶ
        self.distance_to(*point)
    }
}

// ============================================================================
// Note: 他の形状との衝突判定は後続の実装で追加予定
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
    use geo_contracts::BasicCollision; // トレイトを明示的にインポート

    #[test]
    fn test_torus_surface_point_on_surface() {
        let torus = TorusSurface3D::standard(5.0, 2.0).unwrap();

        // トーラス表面上の点（パラメトリック計算）
        let u = 0.0_f64;
        let v = 0.0_f64;
        let point = torus.point_at(u, v);

        let distance = BasicCollision::distance_to(&torus, &point);
        assert!(distance < 1e-10, "Expected ~0, got {}", distance);
    }

    #[test]
    fn test_torus_surface_point_inside() {
        let torus = TorusSurface3D::standard(5.0, 2.0).unwrap();

        // トーラス管の内部の点
        let point = Point3D::new(5.0, 0.0, 0.0); // 中心円上
        let distance = BasicCollision::distance_to(&torus, &point);

        // 副半径2.0なので、中心円から表面まで2.0の距離
        assert!(
            (distance - 2.0).abs() < 1e-10,
            "Expected ~2.0, got {}",
            distance
        );
    }

    #[test]
    fn test_torus_surface_intersects() {
        let torus = TorusSurface3D::standard(5.0, 2.0).unwrap();

        // 表面上の点
        let point_on_surface = torus.point_at(0.0, 0.0);
        assert!(torus.intersects(&point_on_surface, 1e-6));

        // 遠い点
        let far_point = Point3D::new(100.0, 100.0, 100.0);
        assert!(!torus.intersects(&far_point, 1e-6));
    }
}
