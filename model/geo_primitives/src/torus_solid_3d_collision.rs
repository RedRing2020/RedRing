//! TorusSolid3D の衝突判定実装
//!
//! トーラス立体との衝突判定（含内部）を提供する。
//! トーラス立体は内部を持つ立体であり、距離計算は表面までの距離または内部からの距離を返す。

use crate::{Point3D, TorusSolid3D};
use geo_contracts::BasicCollision;
use geo_contracts::Scalar;

// ============================================================================
// BasicCollision implementations for TorusSolid3D
// ============================================================================

/// TorusSolid3D と Point3D の衝突判定
impl<T: Scalar> BasicCollision<T, Point3D<T>> for TorusSolid3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // Solid の場合、overlaps は内部または表面上にあることを意味
        self.distance_to_point(point) <= tolerance
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // extensions の distance_to_point を呼ぶ
        self.distance_to_point(point)
    }
}

// ============================================================================
// Note: 他の形状との衝突判定は後続の実装で追加予定
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
    use geo_contracts::BasicCollision; // トレイトを明示的にインポート

    #[test]
    fn test_torus_solid_point_on_surface() {
        let torus = TorusSolid3D::standard(5.0, 2.0).unwrap();

        // トーラス表面上の点（パラメトリック計算）
        // u=0, v=0 の場合: (major_radius + minor_radius, 0, 0)
        let point = Point3D::new(7.0, 0.0, 0.0); // 5.0 + 2.0

        let distance = BasicCollision::distance_to(&torus, &point);
        assert!(distance < 1e-10, "Expected ~0, got {}", distance);
    }

    #[test]
    fn test_torus_solid_point_inside() {
        let torus = TorusSolid3D::standard(5.0, 2.0).unwrap();

        // トーラス管の内部の点（中心円から少し内側）
        let point = Point3D::new(5.0, 0.0, 0.0); // 中心円上

        let distance = BasicCollision::distance_to(&torus, &point);
        // 中心円から表面まで2.0（副半径）の距離
        // 内部なので負の値
        assert!(
            (distance + 2.0).abs() < 1e-10,
            "Expected ~-2.0, got {}",
            distance
        );
    }

    #[test]
    fn test_torus_solid_point_outside() {
        let torus = TorusSolid3D::standard(5.0, 2.0).unwrap();

        // トーラスの外部の点
        let point = Point3D::new(20.0, 0.0, 0.0);

        let distance = torus.distance_to(&point);
        // 外径は major_radius + minor_radius = 7.0
        // 20.0 - 7.0 = 13.0
        assert!(
            (distance - 13.0).abs() < 1e-10,
            "Expected ~13.0, got {}",
            distance
        );
    }

    #[test]
    fn test_torus_solid_intersects() {
        let torus = TorusSolid3D::standard(5.0, 2.0).unwrap();

        // 表面上の点（u=0, v=0の場合）
        let point_on_surface = Point3D::new(7.0, 0.0, 0.0);
        assert!(torus.intersects(&point_on_surface, 1e-6));

        // 内部の点
        let point_inside = Point3D::new(4.0, 0.0, 0.0);
        assert!(torus.intersects(&point_inside, 3.0)); // tolerance > distance

        // 遠い点
        let far_point = Point3D::new(100.0, 100.0, 100.0);
        assert!(!torus.intersects(&far_point, 1e-6));
    }
}
