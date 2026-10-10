// torus_surface_3d_extensions.rs
// TorusSurface3D の拡張機能実装
//

use crate::{Point3D, TorusSurface3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> TorusSurface3D<T> {
    /// 指定した点に最も近い表面上の点を探索
    pub fn closest_point_to(&self, target: Point3D<T>) -> Point3D<T> {
        // トーラス表面への最近点探索は非線形最適化問題
        // 初期推定から反復計算で解を求める

        // 1. 原点を基準とした相対位置
        let relative_point = Vector3D::new(
            target.x() - self.origin_internal().x(),
            target.y() - self.origin_internal().y(),
            target.z() - self.origin_internal().z(),
        );

        // 2. 局所座標系への変換
        let x_axis_vec = Vector3D::new(
            self.x_axis_internal().x(),
            self.x_axis_internal().y(),
            self.x_axis_internal().z(),
        );
        let y_axis_vec = Vector3D::new(
            self.y_axis_internal().x(),
            self.y_axis_internal().y(),
            self.y_axis_internal().z(),
        );
        let z_axis_vec = Vector3D::new(
            self.z_axis_internal().x(),
            self.z_axis_internal().y(),
            self.z_axis_internal().z(),
        );

        let local_x = relative_point.dot(&x_axis_vec);
        let local_y = relative_point.dot(&y_axis_vec);
        let local_z = relative_point.dot(&z_axis_vec);

        // 3. 主方向角度 u の初期推定
        let u_initial = local_y.atan2(local_x);

        // 4. 管の中心線への投影
        let radial_distance = (local_x * local_x + local_y * local_y).sqrt();
        let projected_radius = if radial_distance > T::EPSILON {
            radial_distance
        } else {
            self.major_radius_internal() // 中心軸上の場合
        };

        // 5. 副方向角度 v の初期推定
        let tube_center_distance = projected_radius - self.major_radius_internal();
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
        diff.magnitude()
    }

    /// 主曲率を計算
    ///
    /// # Arguments
    /// * `u` - 主方向パラメータ
    /// * `v` - 副方向パラメータ
    ///
    /// # Returns
    /// (主曲率1, 主曲率2) のタプル
    pub fn principal_curvatures(&self, _u: T, v: T) -> (T, T) {
        let cos_v = v.cos();

        // 主曲率の計算
        let k1 = cos_v / (self.major_radius_internal() + self.minor_radius_internal() * cos_v);
        let k2 = T::ONE / self.minor_radius_internal();

        (k1, k2)
    }

    /// ガウス曲率を計算
    ///
    /// 表面の幾何学的性質を特徴づける重要な量です。
    pub fn gaussian_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        k1 * k2
    }

    /// 平均曲率を計算
    pub fn mean_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        let two = T::ONE + T::ONE;
        (k1 + k2) / two
    }
}

#[cfg(test)]
mod tests {
    use crate::TorusSurface3D;
    use analysis::test_constants::TOLERANCE_F64;
    use std::f64::consts::{FRAC_PI_2, PI};

    const MAJOR_RADIUS: f64 = 5.0;
    const MINOR_RADIUS: f64 = 1.0;

    fn torus() -> TorusSurface3D<f64> {
        TorusSurface3D::standard(MAJOR_RADIUS, MINOR_RADIUS).unwrap()
    }

    #[test]
    fn gaussian_curvature_is_positive_on_outer_equator() {
        // 外側赤道（v = 0）は楕円点: K = 1 / (r (R + r))
        let expected = 1.0 / (MINOR_RADIUS * (MAJOR_RADIUS + MINOR_RADIUS));
        let actual = torus().gaussian_curvature(0.0, 0.0);
        assert!((actual - expected).abs() < TOLERANCE_F64);
    }

    #[test]
    fn gaussian_curvature_is_negative_on_inner_equator() {
        // 内側赤道（v = π）は双曲点: K = -1 / (r (R - r))
        let expected = -1.0 / (MINOR_RADIUS * (MAJOR_RADIUS - MINOR_RADIUS));
        let actual = torus().gaussian_curvature(0.0, PI);
        assert!((actual - expected).abs() < TOLERANCE_F64);
    }

    #[test]
    fn gaussian_curvature_is_zero_on_top_and_bottom_circles() {
        // 上下の円（v = ±π/2）は放物点: K = 0
        let torus = torus();
        assert!(torus.gaussian_curvature(0.0, FRAC_PI_2).abs() < TOLERANCE_F64);
        assert!(torus.gaussian_curvature(0.0, -FRAC_PI_2).abs() < TOLERANCE_F64);
    }

    #[test]
    fn gaussian_curvature_is_independent_of_u_and_equals_principal_product() {
        // 回転面のため u に依存せず、主曲率の積に一致する
        let torus = torus();
        let v = 0.7;
        let reference = torus.gaussian_curvature(0.0, v);
        for u in [0.5, 1.0, PI, 4.0] {
            assert!((torus.gaussian_curvature(u, v) - reference).abs() < TOLERANCE_F64);
        }
        let (k1, k2) = torus.principal_curvatures(1.0, v);
        assert!((reference - k1 * k2).abs() < TOLERANCE_F64);
    }
}
