//! `NurbsCurve3D` の Foundation パターン実装

use crate::NurbsCurve3D;
use analysis::Point3;
use geo_core::Aabb3D;
use geo_foundation::{Bounded, ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for NurbsCurve3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsCurve3D
    }

    /// 曲線の測度（曲線長）を返す
    ///
    /// # 注意
    /// 近似値を返します（100分割でのサンプリング）
    /// より精密な計算が必要な場合は `approximate_length()` を直接使用
    fn measure(&self) -> Option<T> {
        Some(self.approximate_length(100))
    }
}

impl<T: Scalar> Bounded<T> for NurbsCurve3D<T> {
    type Aabb = Aabb3D<T>;

    /// 境界ボックスを返す（制御点ベース・高速）
    ///
    /// NURBS曲線の凸包性質を利用: 曲線は必ず制御点の凸包内に収まる
    /// このため、制御点の境界ボックスは常に曲線を含む保守的な推定となる
    ///
    /// # 計算量
    /// O(n) - n は制御点数
    ///
    /// # 精度
    /// 保守的（常に曲線を含むが、余分な空間を含む可能性がある）
    /// 精密な境界ボックスが必要な場合は `precise_bounding_box()` を使用
    fn aabb(&self) -> Option<Self::Aabb> {
        debug_assert!(self.num_points() > 0, "Control points should not be empty");
        let coords = self.coordinates();
        let mut points = Vec::with_capacity(self.num_points());
        for i in 0..self.num_points() {
            let base = i * 3;
            points.push(Point3::new(
                coords[base],
                coords[base + 1],
                coords[base + 2],
            ));
        }
        Aabb3D::from_points(&points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clamped_knot_vector;
    use analysis::linalg::vector::Vector3;
    use geo_foundation::{Bounded, ExtensionFoundation};

    #[test]
    fn test_nurbs_curve_3d_foundation() {
        // 簡単なNURBS曲線を作成
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
        ];

        let knots = clamped_knot_vector(2, 3);
        let curve = NurbsCurve3D::new(control_points, None, knots, 2).unwrap();

        // PrimitiveKind の確認
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);

        // AABB の確認
        let bbox = curve.aabb().expect("NURBS curve should have bounding box");
        let min = bbox.min();
        let max = bbox.max();

        // 境界ボックスは制御点を含む
        assert!((min.x - 0.0).abs() < 1e-10);
        assert!((min.y - 0.0).abs() < 1e-10);
        assert!((min.z - 0.0).abs() < 1e-10);
        assert!((max.x - 2.0).abs() < 1e-10);
        assert!((max.y - 1.0).abs() < 1e-10);
        assert!((max.z - 0.0).abs() < 1e-10);

        // 測度（曲線長）の確認
        let length = curve.measure();
        assert!(length.is_some());
        assert!(length.unwrap() > 0.0);
    }
}
