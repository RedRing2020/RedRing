//! TorusSurface3D の拡張機能実装

use crate::{Point3D, TorusSurface3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> TorusSurface3D<T> {
    /// 指定した点に最も近い表面上の点を探索
    pub fn closest_point_to(&self, target: Point3D<T>) -> Point3D<T> {
        // トーラス表面への最近点探索は非線形最適化問題
        // 初期推定から反復計算で解を求める

        // 1. 原点を基準とした相対位置
        let relative_point = Vector3D::new(
            target.x() - self.origin.x(),
            target.y() - self.origin.y(),
            target.z() - self.origin.z(),
        );

        // 2. 局所座標系への変換
        let x_axis_vec = Vector3D::new(self.x_axis.x(), self.x_axis.y(), self.x_axis.z());
        let y_axis_vec = Vector3D::new(
            self.y_axis_internal().x(),
            self.y_axis_internal().y(),
            self.y_axis_internal().z(),
        );
        let z_axis_vec = Vector3D::new(self.z_axis.x(), self.z_axis.y(), self.z_axis.z());

        let local_x = relative_point.dot(x_axis_vec);
        let local_y = relative_point.dot(y_axis_vec);
        let local_z = relative_point.dot(z_axis_vec);

        // 3. 主方向角度 u の初期推定
        let u_initial = local_y.atan2(local_x);

        // 4. 大半径の円への投影
        let radial_distance = (local_x * local_x + local_y * local_y).sqrt();
        let projected_radius = if radial_distance > T::EPSILON {
            radial_distance
        } else {
            self.major_radius // 中心軸上の場合
        };

        // 5. 副方向角度 v の初期推定
        let tube_center_distance = projected_radius - self.major_radius;
        let v_initial = local_z.atan2(tube_center_distance);
        let _radial_distance_norm = (local_x * local_x + local_y * local_y).sqrt();

        // 6. 表面上の点を計算（最適化結果として初期推定値を使用）
        self.point_at(u_initial, v_initial)
    }

    /// 指定した点への距離を計算
    pub fn distance_to(&self, point: Point3D<T>) -> T {
        let closest = self.closest_point_to(point);
        let diff = Vector3D::new(
            point.x() - closest.x(),
            point.y() - closest.y(),
            point.z() - closest.z(),
        );
        diff.length()
    }
}
