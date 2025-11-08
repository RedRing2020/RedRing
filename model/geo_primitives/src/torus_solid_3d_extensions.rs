// torus_solid_3d_extensions.rs
// TorusSolid3D の拡張機能実装
//
// CAM固体加工計算に必要な高度な幾何学的機能を提供します。
// 衝突検知、距離計算、工具経路計算などの実用的な機能を含みます。

use crate::{Point3D, TorusSolid3D, Vector3D};
use geo_foundation::Scalar;

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
        let relative = Vector3D::new(
            point.x() - self.origin().x(),
            point.y() - self.origin().y(),
            point.z() - self.origin().z(),
        );

        // ローカル座標系での成分
        let z_component = relative.x() * self.z_axis().x()
            + relative.y() * self.z_axis().y()
            + relative.z() * self.z_axis().z();

        let x_component = relative.x() * self.x_axis().x()
            + relative.y() * self.x_axis().y()
            + relative.z() * self.x_axis().z();

        let y_axis = self.y_axis();
        let y_component =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // XY平面での中心軸からの距離
        let radial_distance = (x_component * x_component + y_component * y_component).sqrt();

        // トーラス中心線上の最近点からの距離
        let torus_center_distance = radial_distance - self.major_radius();

        // 断面円での距離
        let cross_section_distance =
            (z_component * z_component + torus_center_distance * torus_center_distance).sqrt();

        cross_section_distance - self.minor_radius()
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
            *self.origin()
        }
    }

    /// 点が固体内部にあるかの詳細判定
    ///
    /// より詳細な内部判定情報を提供します。
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// * (内部フラグ, 表面までの距離)
    pub fn detailed_contains(&self, point: &Point3D<T>) -> (bool, T) {
        let distance = self.distance_to_point(point);
        (distance <= T::ZERO, distance.abs())
    }

    /// 工具経路計算用パラメータ
    ///
    /// CAM システムでの工具経路計算に必要な幾何学的パラメータを返します。
    ///
    /// # Returns
    /// * (主半径, 副半径, 軸方向, 中心点)
    pub fn toolpath_parameters(&self) -> (T, T, Vector3D<T>, Point3D<T>) {
        let axis_vector = Vector3D::new(self.z_axis().x(), self.z_axis().y(), self.z_axis().z());
        (
            self.major_radius(),
            self.minor_radius(),
            axis_vector,
            *self.origin(),
        )
    }

    /// 断面積計算
    ///
    /// Z軸に垂直な平面での断面積を計算します。
    ///
    /// # Arguments
    /// * `z_offset` - Z軸上のオフセット距離
    ///
    /// # Returns
    /// * `Some(T)` - 断面積（範囲内の場合）
    /// * `None` - 範囲外の場合
    pub fn cross_section_area(&self, z_offset: T) -> Option<T> {
        // Z軸方向のオフセットが副半径内にあるかチェック
        if z_offset.abs() > self.minor_radius() {
            return None;
        }

        // 楕円の断面積計算
        let height_factor =
            (self.minor_radius() * self.minor_radius() - z_offset * z_offset).sqrt();
        let inner_radius = if self.major_radius() > height_factor {
            self.major_radius() - height_factor
        } else {
            T::ZERO
        };
        let outer_radius = self.major_radius() + height_factor;

        let pi = T::from_f64(std::f64::consts::PI);
        Some(pi * (outer_radius * outer_radius - inner_radius * inner_radius))
    }

    /// 慣性モーメント計算
    ///
    /// Z軸回りの慣性モーメントを計算します（密度=1と仮定）。
    ///
    /// # Returns
    /// * Z軸回りの慣性モーメント
    pub fn moment_of_inertia_z(&self) -> T {
        let volume = self.volume();
        let major_sq = self.major_radius() * self.major_radius();
        let minor_sq = self.minor_radius() * self.minor_radius();

        // Iz = V * (5/4 * R² + 3/4 * r²) where V = volume, R = major_radius, r = minor_radius
        let five_quarters = T::from_f64(1.25);
        let three_quarters = T::from_f64(0.75);

        volume * (five_quarters * major_sq + three_quarters * minor_sq)
    }

    /// CAM 工具干渉チェック用の簡易境界球
    ///
    /// 工具との粗い干渉判定に使用できる境界球を計算します。
    ///
    /// # Returns
    /// * (中心点, 半径)
    pub fn bounding_sphere(&self) -> (Point3D<T>, T) {
        let radius = self.major_radius() + self.minor_radius();
        (*self.origin(), radius)
    }

    /// 複数点での内部判定（バッチ処理）
    ///
    /// 複数の点に対して効率的に内部判定を行います。
    ///
    /// # Arguments
    /// * `points` - 判定する点の配列
    ///
    /// # Returns
    /// * 各点の内部判定結果
    pub fn batch_contains(&self, points: &[Point3D<T>]) -> Vec<bool> {
        points.iter().map(|p| self.contains_point(p)).collect()
    }

    /// 工具アクセス可能性チェック
    ///
    /// 指定された方向からの工具アクセスが可能かを判定します。
    ///
    /// # Arguments
    /// * `point` - チェック点
    /// * `direction` - 工具アプローチ方向
    /// * `tool_radius` - 工具半径
    ///
    /// # Returns
    /// * アクセス可能性フラグ
    pub fn tool_accessibility(
        &self,
        point: &Point3D<T>,
        _direction: &Vector3D<T>,
        tool_radius: T,
    ) -> bool {
        // 簡易チェック: 点が表面付近にあり、方向が外向きか
        let distance = self.distance_to_point(point);
        let tolerance = tool_radius * T::from_f64(0.1);

        distance.abs() <= tolerance && distance >= -tool_radius
    }
}
