//! TorusSurface3D の Bounds 実装
//!
//! `Bounded` trait により境界ボックス計算を提供します。

use crate::{Point3D, TorusSurface3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for TorusSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // トーラスの外半径（最大半径）
        let outer_radius = self.major_radius + self.minor_radius;

        // 軸方向の範囲
        let z_extent = self.minor_radius;

        // 局所座標系での境界を計算
        let origin = self.origin;

        // Z軸方向の成分を考慮した境界計算
        let z_axis = self.z_axis;
        let z_vec_x = z_axis.x() * z_extent;
        let z_vec_y = z_axis.y() * z_extent;
        let z_vec_z = z_axis.z() * z_extent;

        // X, Y軸方向の最大範囲を計算
        let x_axis = self.x_axis;
        let y_axis = self.y_axis_internal();

        // 各軸成分の最大値を計算
        let x_max_from_x = x_axis.x().abs() * outer_radius;
        let x_max_from_y = y_axis.x().abs() * outer_radius;
        let x_max_from_z = z_vec_x.abs();
        let x_extent = x_max_from_x + x_max_from_y + x_max_from_z;

        let y_max_from_x = x_axis.y().abs() * outer_radius;
        let y_max_from_y = y_axis.y().abs() * outer_radius;
        let y_max_from_z = z_vec_y.abs();
        let y_extent = y_max_from_x + y_max_from_y + y_max_from_z;

        let z_max_from_x = x_axis.z().abs() * outer_radius;
        let z_max_from_y = y_axis.z().abs() * outer_radius;
        let z_max_from_z = z_vec_z.abs();
        let z_extent_final = z_max_from_x + z_max_from_y + z_max_from_z;

        // 境界ボックスの最小・最大点
        let min_point = Point3D::new(
            origin.x() - x_extent,
            origin.y() - y_extent,
            origin.z() - z_extent_final,
        );

        let max_point = Point3D::new(
            origin.x() + x_extent,
            origin.y() + y_extent,
            origin.z() + z_extent_final,
        );

        Some(Aabb3D::new(min_point, max_point))
    }
}
