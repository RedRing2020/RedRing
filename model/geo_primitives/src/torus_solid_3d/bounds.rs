//! TorusSolid3D の Bounds 実装
//!
//! `Bounded` trait により境界ボックス計算を提供します。

use crate::TorusSolid3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for TorusSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let major_radius = self.major_radius;
        let minor_radius = self.minor_radius;
        let total_radius = major_radius + minor_radius;

        let origin = self.origin;

        // 標準的な軸配置の場合は簡易計算
        let _x_axis = self.x_axis;
        let _y_axis = self.y_axis_internal();
        let z_axis = self.z_axis;

        // 主回転軸（Z軸）が標準軸の場合
        if (z_axis.z() - T::ONE).abs() < T::EPSILON {
            // XY平面でのトーラス：Z方向は小半径のみ
            Some(Aabb3D::new(
                crate::Point3D::new(
                    origin.x() - total_radius,
                    origin.y() - total_radius,
                    origin.z() - minor_radius,
                ),
                crate::Point3D::new(
                    origin.x() + total_radius,
                    origin.y() + total_radius,
                    origin.z() + minor_radius,
                ),
            ))
        } else {
            // 回転されたトーラスの場合：保守的な境界ボックス
            let max_extent = total_radius;
            Some(Aabb3D::new(
                crate::Point3D::new(
                    origin.x() - max_extent,
                    origin.y() - max_extent,
                    origin.z() - max_extent,
                ),
                crate::Point3D::new(
                    origin.x() + max_extent,
                    origin.y() + max_extent,
                    origin.z() + max_extent,
                ),
            ))
        }
    }
}
