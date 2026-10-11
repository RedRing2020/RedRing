//! EllipseArc3D の Bounds 実装

use crate::EllipseArc3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for EllipseArc3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 楕円弧の中心と半径（長半軸）から包含する境界ボックスを計算
        let center = self.center();
        let semi_major = self.semi_major();

        // 中心から長半軸分の範囲を境界ボックスとする。母楕円全体を含む保守的な境界ボックスで、
        // 楕円弧の角度範囲には絞らない
        let min_x = center.x() - semi_major;
        let max_x = center.x() + semi_major;
        let min_y = center.y() - semi_major;
        let max_y = center.y() + semi_major;
        let min_z = center.z() - semi_major;
        let max_z = center.z() + semi_major;

        Some(Aabb3D::new(
            crate::Point3D::new(min_x, min_y, min_z),
            crate::Point3D::new(max_x, max_y, max_z),
        ))
    }
}
