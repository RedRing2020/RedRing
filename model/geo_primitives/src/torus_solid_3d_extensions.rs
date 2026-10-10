// torus_solid_3d_extensions.rs
// TorusSolid3D の拡張機能実装
//
// 距離計算、包含判定、境界球などの機能を含みます。

use crate::{Point3D, TorusSolid3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> TorusSolid3D<T> {
    /// 点との最短距離を計算
    ///
    /// 指定された点からトーラス固体表面までの最短距離を計算します。
    /// 点が内部にある場合は負の値を返します。
    ///
    /// # Arguments
    /// * `point` - 距離を計算する点
    ///
    /// # Returns
    /// * 固体表面までの符号付き距離（内部で負、外部で正）
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        // 原点からの相対位置ベクトル
        let origin = self.origin_internal();
        let relative = Vector3D::new(
            point.x() - origin.x(),
            point.y() - origin.y(),
            point.z() - origin.z(),
        );

        // ローカル座標系での成分
        let z_axis = self.z_axis_internal();
        let z_component =
            relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        let x_axis = self.x_axis_internal();
        let x_component =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();

        let y_axis = self.y_axis_internal();
        let y_component =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // XY平面での中心軸からの距離
        let radial_distance = (x_component * x_component + y_component * y_component).sqrt();

        // トーラス中心線上の最近点からの距離
        let torus_center_distance = radial_distance - self.major_radius_internal();

        // 断面円での距離
        let cross_section_distance =
            (z_component * z_component + torus_center_distance * torus_center_distance).sqrt();

        cross_section_distance - self.minor_radius_internal()
    }

    /// 固体表面の最近点を計算
    ///
    /// 指定された点からトーラス固体表面への最近点を計算します。
    ///
    /// # Arguments
    /// * `point` - 最近点を求める基準点
    ///
    /// # Returns
    /// * 固体表面上の最近点
    pub fn closest_point_on_surface(&self, point: &Point3D<T>) -> Point3D<T> {
        // 外表面を使用して最近点を計算
        if let Some(surface) = self.outer_surface() {
            surface.closest_point_to(*point)
        } else {
            // フォールバック: 単純化した計算
            *self.origin_internal()
        }
    }
}
