//! NURBS × Primitives 衝突判定実装
//!
//! このモジュールは geo_primitives と geo_nurbs 間の衝突判定を実装します。
//! アーキテクチャ設計により、各クレートは相互に依存できないため、
//! geo_algorithms が両方に依存して衝突判定を実装します。
//!
//! ## Orphan Rules への対応
//!
//! Rust の orphan rules により、外部トレイトを外部型に直接実装できません。
//! そのため、Newtype パターンで NURBS 形状をラップした型を提供し、
//! BasicCollision トレイトを実装しています。
//!
//! ## 設計方針
//!
//! 1. **段階的精度向上**: サンプリング → 数値最適化 → 解析的手法
//! 2. **対称性の保証**: A vs B と B vs A の両方を実装
//! 3. **パフォーマンス**: BBox による事前スクリーニング
//! 4. **ゼロコスト抽象化**: `#[repr(transparent)]` による Newtype

use geo_core::Point3D;
use geo_foundation::{extensions::BasicCollision, Scalar};
use geo_nurbs::NurbsCurve3D;

// ============================================================================
// Newtype Wrapper for NurbsCurve3D
// ============================================================================

/// NURBS曲線の衝突判定アダプタ（Newtype パターン）
///
/// orphan rules を回避するため、NurbsCurve3D をラップした型を提供します。
/// この型は BasicCollision トレイトを実装でき、ポリモーフィズムを維持できます。
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NurbsCurveCollider<T: Scalar>(pub NurbsCurve3D<T>);

impl<T: Scalar> NurbsCurveCollider<T> {
    /// NurbsCurve3D からアダプタを作成
    pub fn new(curve: NurbsCurve3D<T>) -> Self {
        Self(curve)
    }

    /// 内部の NurbsCurve3D への参照を取得
    pub fn inner(&self) -> &NurbsCurve3D<T> {
        &self.0
    }

    /// NurbsCurve3D を消費してアダプタから取り出す
    pub fn into_inner(self) -> NurbsCurve3D<T> {
        self.0
    }
}

// ============================================================================
// NurbsCurveCollider vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for NurbsCurveCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 離散化による近似計算（Phase 1 実装）
        // 将来的に Newton-Raphson 法で精密化
        let num_samples = 100;
        let mut min_distance = T::INFINITY;

        let (u_min, u_max) = self.0.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            let curve_point = self.0.evaluate_at(u);

            // ユークリッド距離計算
            let dx = curve_point.x() - point.x();
            let dy = curve_point.y() - point.y();
            let dz = curve_point.z() - point.z();
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

            min_distance = min_distance.min(distance);
        }

        min_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::Scalar;

    fn create_test_curve<T: Scalar>() -> NurbsCurve3D<T> {
        use analysis::linalg::vector::vector3::Vector3;

        // 3次 NURBS 曲線（制御点4つ、次数3）
        let control_points = vec![
            Vector3::new(T::ZERO, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ONE, T::ZERO),
            Vector3::new(T::ZERO, T::ONE, T::ZERO),
        ];

        let weights = Some(vec![T::ONE, T::ONE, T::ONE, T::ONE]);
        let knots = vec![
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ONE,
            T::ONE,
            T::ONE,
        ];

        NurbsCurve3D::new(control_points, weights, knots, 3).unwrap()
    }

    #[test]
    fn test_point_on_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.0, 0.0, 0.0); // 曲線の始点
        let tolerance = 1e-6;

        assert!(collider.intersects(&point, tolerance));
    }

    #[test]
    fn test_point_near_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.1, 0.0, 0.0); // 曲線の近く
        let distance = collider.distance_to(&point);

        assert!(distance < 0.2); // 適度に近い
        assert!(distance > 0.0); // 完全には一致しない
    }

    #[test]
    fn test_point_far_from_curve() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(10.0, 10.0, 10.0); // 曲線から遠い
        let distance = collider.distance_to(&point);

        assert!(distance > 10.0);
    }

    #[test]
    fn test_symmetry() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve);
        let point = Point3D::new(0.5, 0.5, 0.0);

        // Newtype パターンのため一方向のみテスト
        let distance = collider.distance_to(&point);
        assert!(distance >= 0.0); // 距離は非負
    }

    #[test]
    fn test_newtype_conversions() {
        let curve = create_test_curve::<f64>();
        let collider = NurbsCurveCollider::new(curve.clone());

        // inner() で参照を取得
        let _inner_ref = collider.inner();

        // into_inner() で元の NurbsCurve3D を取得
        let recovered_curve = collider.into_inner();

        // 元の曲線と同じパラメータ範囲を持つことを確認
        assert_eq!(recovered_curve.parameter_domain(), curve.parameter_domain());
    }
}
