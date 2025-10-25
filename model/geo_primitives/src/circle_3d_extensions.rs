//! Circle3D Extensions - Advanced geometric operations and calculations
//!
//! 3次元円の拡張メソッド：軸取得、点の計算、距離計算、平面基底計算など

use crate::{Circle3D, Direction3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

impl<T: Scalar> Circle3D<T> {
    /// 円平面のU軸（基準軸）を取得
    ///
    /// # 戻り値
    /// 円の平面内での第一基底ベクトル（正規化済み）
    pub fn u_axis(&self) -> Direction3D<T> {
        let (u, _) = self.get_plane_basis();
        Direction3D::from_vector(u).unwrap()
    }

    /// 円平面のV軸を取得
    ///
    /// # 戻り値
    /// 円の平面内での第二基底ベクトル（正規化済み）
    pub fn v_axis(&self) -> Direction3D<T> {
        let (_, v) = self.get_plane_basis();
        Direction3D::from_vector(v).unwrap()
    }

    /// 円上の点を角度から取得
    ///
    /// # 引数
    /// * `angle` - 角度（ラジアン）
    ///
    /// # 戻り値
    /// 円上の点（円の平面内での極座標から3D座標に変換）
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        // 円の平面内での基準ベクトルを計算
        // 法線ベクトルに垂直な2つのベクトルを求める
        let (u, v) = self.get_plane_basis();

        let x = angle.cos();
        let y = angle.sin();

        // 円上の点 = 中心 + radius * (x * u + y * v)
        let offset = Vector3D::new(
            self.radius() * (x * u.x() + y * v.x()),
            self.radius() * (x * u.y() + y * v.y()),
            self.radius() * (x * u.z() + y * v.z()),
        );

        Point3D::new(
            self.center().x() + offset.x(),
            self.center().y() + offset.y(),
            self.center().z() + offset.z(),
        )
    }

    /// 円の平面における基準ベクトル（u, v）を取得
    /// 法線ベクトルに垂直な正規直交基底
    pub fn get_plane_basis(&self) -> (Vector3D<T>, Vector3D<T>) {
        // Z軸方向の法線の場合は特別扱い（XY平面）
        if (self.normal().z() - T::ONE).abs() < DefaultTolerances::distance::<T>() {
            // XY平面：X軸とY軸を使用
            return (Vector3D::unit_x(), Vector3D::unit_y());
        }

        // Y軸方向の法線の場合（XZ平面）
        if (self.normal().y() - T::ONE).abs() < DefaultTolerances::distance::<T>() {
            return (Vector3D::unit_x(), Vector3D::unit_z());
        }

        // X軸方向の法線の場合（YZ平面）
        if (self.normal().x() - T::ONE).abs() < DefaultTolerances::distance::<T>() {
            return (Vector3D::unit_y(), Vector3D::unit_z());
        }

        // 一般的な場合：Gram-Schmidt 過程で正規直交基底を作成
        let temp = if self.normal().z().abs() < DefaultTolerances::distance::<T>() {
            Vector3D::unit_z()
        } else {
            Vector3D::unit_x()
        };

        // 第一基底ベクトル: normal × temp を正規化
        let first_unnormalized = self.normal().as_vector().cross(&temp);
        let first = first_unnormalized.normalize();

        // 第二基底ベクトル: normal × first
        let second = self.normal().as_vector().cross(&first);

        (first, second)
    }

    /// 点が円の平面上にあるかを判定
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// 点が円の平面上にある場合は `true`
    pub fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let center_to_point = Vector3D::from_points(&self.center(), point);
        let distance_to_plane = center_to_point.dot(&self.normal().as_vector()).abs();
        distance_to_plane <= tolerance
    }

    /// 点から円の中心への距離（3D空間内）
    ///
    /// # 引数
    /// * `point` - 距離を計算する点
    ///
    /// # 戻り値
    /// 3D空間での直線距離
    pub fn distance_to_center(&self, point: &Point3D<T>) -> T {
        self.center().distance_to(point)
    }

    /// 点から円への最短距離
    ///
    /// # 引数
    /// * `point` - 距離を計算する点
    ///
    /// # 戻り値
    /// 点から円周上の最近点への3D距離
    pub fn distance_to_circle(&self, point: &Point3D<T>) -> T {
        // 点を円の平面に投影
        let center_to_point = Vector3D::from_points(&self.center(), point);
        let plane_distance = center_to_point.dot(&self.normal().as_vector());

        // 平面上での投影点
        let projected_offset = Vector3D::new(
            center_to_point.x() - plane_distance * self.normal().x(),
            center_to_point.y() - plane_distance * self.normal().y(),
            center_to_point.z() - plane_distance * self.normal().z(),
        );

        let radial_distance = projected_offset.length();
        let circle_distance = (radial_distance - self.radius()).abs();

        // 3D距離 = √(平面距離² + 円距離²)
        (plane_distance * plane_distance + circle_distance * circle_distance).sqrt()
    }

    /// 円周上の等間隔な点列を生成
    ///
    /// # 引数
    /// * `num_points` - 生成する点の数
    ///
    /// # 戻り値
    /// 円周上の等間隔な点のベクトル
    pub fn sample_points(&self, num_points: usize) -> Vec<Point3D<T>> {
        if num_points == 0 {
            return Vec::new();
        }

        let mut points = Vec::with_capacity(num_points);

        // usizeをT型に変換（手動実装）
        let mut num_points_scalar = T::ZERO;
        for _ in 0..num_points {
            num_points_scalar += T::ONE;
        }

        let angle_step = T::TAU / num_points_scalar;

        for i in 0..num_points {
            // iをT型に変換
            let mut i_scalar = T::ZERO;
            for _ in 0..i {
                i_scalar += T::ONE;
            }

            let angle = i_scalar * angle_step;
            points.push(self.point_at_angle(angle));
        }

        points
    }

    /// 円が含まれる平面の方程式を取得
    ///
    /// # 戻り値
    /// 平面の方程式: (法線ベクトル, 平面上の点からの距離)
    /// ax + by + cz + d = 0 の形で、(a, b, c) = 法線ベクトル、d = -法線·中心点
    pub fn plane_equation(&self) -> (Vector3D<T>, T) {
        let normal = self.normal().as_vector();
        let d = -normal.dot(&Vector3D::new(
            self.center().x(),
            self.center().y(),
            self.center().z(),
        ));
        (normal, d)
    }

    /// 指定角度での接線ベクトルを取得
    ///
    /// # 引数
    /// * `angle` - 角度（ラジアン）
    ///
    /// # 戻り値
    /// 指定角度での接線方向ベクトル（正規化済み）
    pub fn tangent_at_angle(&self, angle: T) -> Direction3D<T> {
        let (u, v) = self.get_plane_basis();

        // 接線ベクトル = -sin(θ) * u + cos(θ) * v
        let tangent = Vector3D::new(
            -angle.sin() * u.x() + angle.cos() * v.x(),
            -angle.sin() * u.y() + angle.cos() * v.y(),
            -angle.sin() * u.z() + angle.cos() * v.z(),
        );

        Direction3D::from_vector(tangent).unwrap()
    }

    /// 点から円への最近点を取得
    ///
    /// # 引数
    /// * `point` - 対象点
    ///
    /// # 戻り値
    /// 円周上の最近点
    pub fn closest_point_on_circle(&self, point: &Point3D<T>) -> Point3D<T> {
        let center_to_point = Vector3D::from_points(&self.center(), point);
        let plane_distance = center_to_point.dot(&self.normal().as_vector());

        // 平面上での投影点への方向
        let projected_direction = Vector3D::new(
            center_to_point.x() - plane_distance * self.normal().x(),
            center_to_point.y() - plane_distance * self.normal().y(),
            center_to_point.z() - plane_distance * self.normal().z(),
        );

        let radial_distance = projected_direction.length();
        if radial_distance == T::ZERO {
            // 点が中心にある場合、任意の円周上の点を返す
            return self.point_at_angle(T::ZERO);
        }

        // 正規化された方向ベクトル
        let normalized_direction = projected_direction.normalize();

        // 円周上の最近点
        Point3D::new(
            self.center().x() + self.radius() * normalized_direction.x(),
            self.center().y() + self.radius() * normalized_direction.y(),
            self.center().z() + self.radius() * normalized_direction.z(),
        )
    }
}
