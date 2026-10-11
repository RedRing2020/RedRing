//! `NurbsSurface3D` の Bounds 実装
//!
//! `Bounded` trait を提供する。

use crate::NurbsSurface3D;
use crate::Scalar;
use geo_contracts::Bounded;

impl<T: Scalar> Bounded<T> for NurbsSurface3D<T> {
    type Aabb = geo_core::Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 制御点ベースの境界ボックスを計算
        let mut min_x = T::from_f64(f64::INFINITY);
        let mut min_y = T::from_f64(f64::INFINITY);
        let mut min_z = T::from_f64(f64::INFINITY);
        let mut max_x = T::from_f64(f64::NEG_INFINITY);
        let mut max_y = T::from_f64(f64::NEG_INFINITY);
        let mut max_z = T::from_f64(f64::NEG_INFINITY);

        for u in 0..self.grid_size().0 {
            for v in 0..self.grid_size().1 {
                let point = self.control_point(u, v);
                min_x = min_x.min(point.x());
                min_y = min_y.min(point.y());
                min_z = min_z.min(point.z());
                max_x = max_x.max(point.x());
                max_y = max_y.max(point.y());
                max_z = max_z.max(point.z());
            }
        }

        Some(geo_core::Aabb3D::new(
            geo_core::Point3D::new(min_x, min_y, min_z),
            geo_core::Point3D::new(max_x, max_y, max_z),
        ))
    }
}
