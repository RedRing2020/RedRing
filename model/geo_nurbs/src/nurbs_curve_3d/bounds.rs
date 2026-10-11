//! `NurbsCurve3D` の Bounds 実装

use crate::NurbsCurve3D;
use geo_contracts::Bounded;
use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};

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
        let mut points = Vec::new();
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
