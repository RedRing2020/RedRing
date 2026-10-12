//! ConicalSurface3D の拡張機能実装
//!
//! 基本機能を超えた幾何操作・解析機能

use crate::{ConicalSurface3D, Point3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> ConicalSurface3D<T> {
    /// 点から円錐サーフェス上の最近点を求める
    ///
    /// # Returns
    /// * `Point3D<T>` - 最近点
    /// * `(T, T)` - (u, v) パラメータ
    /// * `T` - 距離
    pub fn closest_point_to(&self, point: &Point3D<T>) -> (Point3D<T>, (T, T), T) {
        // 簡略実装：軸方向を固定して放射方向の最短点を求める
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let z_axis = self.axis.as_vector();
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis_internal().as_vector();

        // 軸方向成分
        let v = relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        // 放射方向成分
        let radial_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let radial_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // 角度パラメータ u を計算
        let u = radial_y.atan2(radial_x);
        let u = if u < T::ZERO {
            u + T::PI * (T::ONE + T::ONE)
        } else {
            u
        };

        // サーフェス上の点
        let surface_point = self.point_at_uv(u, v);
        let distance = self.distance_to_surface(point);

        (surface_point, (u, v), distance)
    }
}
