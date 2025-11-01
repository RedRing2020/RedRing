// torus_surface_3d_extensions.rs
// TorusSurface3D の拡張機能実装
//
// 3D CAM 工具オフセット計算を含む高度な幾何計算機能を提供します。

use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
use geo_foundation::Scalar;
use std::f64::consts::PI;

impl<T: Scalar> TorusSurface3D<T> {
    /// 指定した点に最も近い表面上の点を探索
    ///
    /// 3D CAM での工具パス計算において、工具中心から表面への
    /// 最短距離計算に使用されます。
    pub fn closest_point_to(&self, target: Point3D<T>) -> Point3D<T> {
        // トーラス表面への最近点探索は非線形最適化問題
        // 初期推定から反復計算で解を求める

        // 1. 原点を基準とした相対位置
        let relative_point = Vector3D::new(
            target.x() - self.origin().x(),
            target.y() - self.origin().y(),
            target.z() - self.origin().z(),
        );

        // 2. 局所座標系への変換
        let x_axis_vec = Vector3D::new(self.x_axis().x(), self.x_axis().y(), self.x_axis().z());
        let y_axis_vec = Vector3D::new(self.y_axis().x(), self.y_axis().y(), self.y_axis().z());
        let z_axis_vec = Vector3D::new(self.z_axis().x(), self.z_axis().y(), self.z_axis().z());

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
            self.major_radius() // 中心軸上の場合
        };

        // 5. 副方向角度 v の初期推定
        let tube_center_distance = projected_radius - self.major_radius();
        let v_initial = local_z.atan2(tube_center_distance);
        let _radial_distance_norm = (local_x * local_x + local_y * local_y).sqrt();

        // 6. 表面上の点を計算（最適化結果として初期推定値を使用）
        self.point_at(u_initial, v_initial)
    }

    /// 指定した点への距離を計算
    ///
    /// CAM での工具オフセット量の決定に使用されます。
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
    ///
    /// CAM での工具選択と送り速度決定に重要な情報です。
    pub fn principal_curvatures(&self, _u: T, v: T) -> (T, T) {
        let cos_v = v.cos();

        // 主曲率の計算
        let k1 = cos_v / (self.major_radius() + self.minor_radius() * cos_v);
        let k2 = T::ONE / self.minor_radius();

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
    ///
    /// 表面の滑らかさの指標として CAM での仕上げ条件決定に使用されます。
    pub fn mean_curvature(&self, u: T, v: T) -> T {
        let (k1, k2) = self.principal_curvatures(u, v);
        let two = T::ONE + T::ONE;
        (k1 + k2) / two
    }

    /// 工具パス計算用のパラメータを取得
    ///
    /// 3D CAM での工具パス生成に必要な幾何学的パラメータを返します。
    ///
    /// # Arguments
    /// * `u` - 主方向パラメータ
    /// * `v` - 副方向パラメータ
    /// * `tool_radius` - 工具半径
    ///
    /// # Returns
    /// (工具中心位置, 送り方向, 法線方向, 推奨送り速度係数)
    pub fn toolpath_parameters(
        &self,
        u: T,
        v: T,
        tool_radius: T,
    ) -> (Point3D<T>, Direction3D<T>, Direction3D<T>, T) {
        let surface_point = self.point_at(u, v);
        let surface_normal = self.normal_at(u, v);

        // 工具中心位置（表面から工具半径分オフセット）
        let tool_center = Point3D::new(
            surface_point.x() + surface_normal.x() * tool_radius,
            surface_point.y() + surface_normal.y() * tool_radius,
            surface_point.z() + surface_normal.z() * tool_radius,
        );

        // 送り方向（u方向の接線）
        let du = T::from_f64(0.001); // 小さな増分
        let u_plus = u + du;
        let point_u_plus = self.point_at(u_plus, v);
        let feed_direction_vec = Vector3D::new(
            point_u_plus.x() - surface_point.x(),
            point_u_plus.y() - surface_point.y(),
            point_u_plus.z() - surface_point.z(),
        );
        let feed_direction = Direction3D::from_vector(feed_direction_vec).unwrap_or(self.x_axis()); // フォールバック

        // 曲率に基づく送り速度係数
        let mean_curvature = self.mean_curvature(u, v);
        let curvature_factor = if mean_curvature.abs() > T::EPSILON {
            let curvature_radius = T::ONE / mean_curvature.abs();
            let normalized_radius = curvature_radius / tool_radius;
            // 曲率が大きいほど送り速度を下げる
            (T::ONE / (T::ONE + normalized_radius)).min(T::ONE)
        } else {
            T::ONE // 平坦部分では最大速度
        };

        (
            tool_center,
            feed_direction,
            surface_normal,
            curvature_factor,
        )
    }

    /// 等高線パラメータを計算
    ///
    /// 指定した Z 高さでの等高線パラメータ (u, v) を求めます。
    /// CAM での水平加工に使用されます。
    pub fn contour_parameters_at_height(&self, z_height: T) -> Vec<(T, T)> {
        let mut contours = Vec::new();

        // Z軸方向の成分を考慮した高さ計算
        let origin_z = self.origin().z();
        let relative_height = z_height - origin_z;

        // 局所座標での Z 高さに対応する v パラメータを求める
        let max_z_extent = self.minor_radius();

        if relative_height.abs() <= max_z_extent {
            // v パラメータを計算（近似）
            let v_value = (relative_height / self.minor_radius()).asin();

            // u パラメータは 0 から 2π まで一周
            let num_points = 64; // 分割数
            let du = T::from_f64(2.0 * PI) / T::from_f64(num_points as f64);

            for i in 0..num_points {
                let u_value = T::from_f64(i as f64) * du;
                contours.push((u_value, v_value));
            }
        }

        contours
    }

    /// トーラス面のパラメータ範囲を取得
    ///
    /// # Returns
    /// ((u_min, u_max), (v_min, v_max))
    pub fn parameter_bounds(&self) -> ((T, T), (T, T)) {
        let zero = T::ZERO;
        let two_pi = T::from_f64(2.0 * PI);

        ((zero, two_pi), (zero, two_pi))
    }

    /// 表面の局所座標系を取得
    ///
    /// 指定したパラメータ位置での局所座標系（接平面基底）を返します。
    pub fn local_coordinate_system(
        &self,
        u: T,
        v: T,
    ) -> (Direction3D<T>, Direction3D<T>, Direction3D<T>) {
        let normal = self.normal_at(u, v);

        // u方向の接線ベクトル
        let du = T::from_f64(0.001);
        let point_current = self.point_at(u, v);
        let point_u_plus = self.point_at(u + du, v);
        let u_tangent_vec = Vector3D::new(
            point_u_plus.x() - point_current.x(),
            point_u_plus.y() - point_current.y(),
            point_u_plus.z() - point_current.z(),
        );
        let u_tangent = Direction3D::from_vector(u_tangent_vec).unwrap_or(self.x_axis());

        // v方向の接線ベクトル（u_tangent と normal の外積）
        let normal_vec = Vector3D::new(normal.x(), normal.y(), normal.z());
        let u_tangent_vec_norm = Vector3D::new(u_tangent.x(), u_tangent.y(), u_tangent.z());
        let v_tangent_vec = normal_vec.cross(&u_tangent_vec_norm);
        let v_tangent = Direction3D::from_vector(v_tangent_vec).unwrap_or(self.y_axis());

        (u_tangent, v_tangent, normal)
    }
}

/// f64 専用の高精度計算機能
impl TorusSurface3D<f64> {
    /// 高精度な最近点探索（Newton-Raphson法）
    ///
    /// CAM での高精度な工具パス計算に使用されます。
    pub fn closest_point_precise(&self, target: Point3D<f64>) -> Point3D<f64> {
        // より高精度な計算が必要な場合はここで Newton-Raphson 法を実装
        // 現在は基本実装を返す
        self.closest_point_to(target)
    }

    /// 工具干渉チェック
    ///
    /// 指定した工具が表面と干渉するかどうかをチェックします。
    pub fn tool_interference_check(&self, tool_center: Point3D<f64>, tool_radius: f64) -> bool {
        let distance = self.distance_to(tool_center);
        distance < tool_radius
    }

    /// 推奨加工パラメータを計算
    ///
    /// 表面の幾何学的特性に基づいて推奨される加工パラメータを返します。
    pub fn recommended_machining_parameters(
        &self,
        u: f64,
        v: f64,
        tool_radius: f64,
    ) -> (f64, f64, f64) {
        let (k1, k2) = self.principal_curvatures(u, v);
        let mean_curvature = (k1 + k2) / 2.0;

        // 推奨送り速度（曲率に基づく）
        let feed_rate = if mean_curvature.abs() > 1e-6 {
            let curvature_radius = 1.0 / mean_curvature.abs();
            (tool_radius / curvature_radius).clamp(0.1, 1.0)
        } else {
            1.0
        };

        // 推奨主軸回転数（表面粗さ考慮）
        let spindle_speed = 1000.0 / (tool_radius + 0.1);

        // 推奨切込み深さ
        let depth_of_cut = tool_radius * 0.1;

        (feed_rate, spindle_speed, depth_of_cut)
    }
}
