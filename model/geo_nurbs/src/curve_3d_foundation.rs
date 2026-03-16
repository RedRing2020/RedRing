//! `NurbsCurve3D` の Foundation パターン実装

use crate::NurbsCurve3D;
use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};
use geo_foundation::{Bounded, ExtensionFoundation, PrimitiveKind};

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
            points.push(Point3D::new(
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
    use geo_foundation::{Bounded, ExtensionFoundation};

    #[test]
    fn test_nurbs_curve_3d_foundation() {
        use geo_foundation::NurbsCurve3DConstructor;
        // 簡単なNURBS曲線を作成
        let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)];

        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            2,
            knots,
            control_points,
            None,
        )
        .unwrap();

        // PrimitiveKind の確認
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);

        // AABB の確認
        let bbox = curve.aabb().expect("NURBS curve should have bounding box");
        let min = bbox.min();
        let max = bbox.max();

        // 境界ボックスは制御点を含む
        assert!((min.x() - 0.0).abs() < 1e-10);
        assert!((min.y() - 0.0).abs() < 1e-10);
        assert!((min.z() - 0.0).abs() < 1e-10);
        assert!((max.x() - 2.0).abs() < 1e-10);
        assert!((max.y() - 1.0).abs() < 1e-10);
        assert!((max.z() - 0.0).abs() < 1e-10);

        // 測度（曲線長）の確認
        let length = curve.measure();
        assert!(length.is_some());
        assert!(length.unwrap() > 0.0);
    }

    #[test]
    fn test_core_traits_constructor_new() {
        use geo_foundation::NurbsCurve3DConstructor;

        // トレイト経由で作成（型を明示）
        let degree = 2;
        let knots = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let control_points = vec![(0.0_f64, 0.0, 0.0), (0.5, 1.0, 0.0), (1.0, 0.0, 0.0)];
        let weights = None;

        let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            degree,
            knots,
            control_points,
            weights,
        );
        assert!(result.is_ok());

        let curve = result.unwrap();
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
    }

    #[test]
    fn test_core_traits_from_bezier() {
        use geo_foundation::NurbsCurve3DConstructor;

        let control_points = vec![(0.0_f64, 0.0, 0.0), (0.5, 1.0, 0.0), (1.0, 0.0, 0.0)];
        let result =
            <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::from_bezier(control_points);
        assert!(result.is_ok());

        let curve = result.unwrap();
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
    }

    #[test]
    fn test_core_traits_line_segment() {
        use geo_foundation::NurbsCurve3DConstructor;

        let start = (0.0_f64, 0.0, 0.0);
        let end = (1.0, 1.0, 1.0);
        let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(start, end);
        assert!(result.is_ok());

        let curve = result.unwrap();
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
    }

    #[test]
    fn test_core_traits_properties() {
        use geo_foundation::{NurbsCurve3DConstructor, NurbsCurve3DProperties};

        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::from_bezier(vec![
            (0.0, 0.0, 0.0),
            (0.5, 1.0, 0.0),
            (1.0, 0.0, 0.0),
        ])
        .unwrap();

        // Propertiesトレイトメソッド確認
        assert_eq!(curve.degree(), 2);
        assert_eq!(curve.control_points_count(), 3);
        assert!(!curve.is_rational());

        let (t_min, t_max) = curve.parameter_domain();
        assert!((t_min - 0.0).abs() < 1e-10);
        assert!((t_max - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_core_traits_measure() {
        use geo_foundation::{NurbsCurve3DConstructor, NurbsCurve3DMeasure};

        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (3.0, 0.0, 0.0),
        )
        .unwrap();

        // Measureトレイトメソッド確認
        let tolerance = 1e-6;
        let total_length = curve.arc_length_total(tolerance);
        assert!((total_length - 3.0).abs() < 1e-3);

        let half_length = curve.arc_length(0.0, 0.5, tolerance);
        assert!((half_length - 1.5).abs() < 1e-3);

        let point_opt = curve.evaluate(0.5);
        assert!(point_opt.is_some());
        let point = point_opt.unwrap();
        assert!((point.0 - 1.5).abs() < 1e-10);
    }
}
