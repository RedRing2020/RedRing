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

    /// 指定点での主曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// (主曲率1, 主曲率2)
    pub fn principal_curvatures(&self, _u: T, v: T) -> (T, T) {
        let r_at_v = self.radius_at_v_internal(v);
        let cos_angle = self.semi_angle.cos();

        // 円錐面の主曲率
        // κ1 = 0 (母線方向：直線のため曲率0)
        // κ2 = cos(α) / r(v) (円周方向)
        let k1 = T::ZERO;
        let k2 = if r_at_v > T::ZERO {
            cos_angle / r_at_v
        } else {
            T::ZERO // 頂点では未定義
        };

        (k1, k2)
    }

    /// 平均曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// 平均曲率 H = (κ1 + κ2) / 2
    pub fn mean_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        (k1 + k2) / (T::ONE + T::ONE)
    }

    /// ガウス曲率を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// ガウス曲率 K = κ1 * κ2
    pub fn gaussian_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        k1 * k2 // 円錐面では常に0（可展面）
    }
}
