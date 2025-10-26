//! 3次元円柱（Cylinder3D）のCore実装
//!
//! Core Foundation パターンに基づく Cylinder3D の必須機能のみ
//! 拡張機能は cylinder_3d_extensions.rs を参照

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元円柱（Core実装）
///
/// 円柱は軸（中心線）、半径、高さで定義される
/// Core機能のみ：
/// - 基本構築・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ（体積、表面積）
/// - 境界ボックス計算
#[derive(Debug, Clone, PartialEq)]
pub struct Cylinder3D<T: Scalar> {
    /// 円柱の底面中心点
    center: Point3D<T>,
    /// 円柱の軸方向ベクトル（正規化されている）
    axis: Vector3D<T>,
    /// 円柱の半径
    radius: T,
    /// 円柱の高さ
    height: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Cylinder3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい円柱を作成
    ///
    /// # Arguments
    /// * `center` - 底面の中心点
    /// * `axis` - 軸方向ベクトル（自動的に正規化される）
    /// * `radius` - 半径（正の値）
    /// * `height` - 高さ（正の値）
    ///
    /// # Returns
    /// 有効な円柱が作成できた場合は `Some(Cylinder3D)`、
    /// 無効なパラメータの場合は `None`
    pub fn new(center: Point3D<T>, axis: Vector3D<T>, radius: T, height: T) -> Option<Self> {
        // 半径と高さの検証
        if radius <= T::ZERO || height <= T::ZERO {
            return None;
        }

        // 軸ベクトルの正規化
        let normalized_axis = axis.normalize();
        if normalized_axis == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        Some(Self {
            center,
            axis: normalized_axis,
            radius,
            height,
        })
    }

    /// Z軸に平行な円柱を作成（簡易コンストラクタ）
    pub fn new_z_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            radius,
            height,
        )
    }

    /// Y軸に平行な円柱を作成（簡易コンストラクタ）
    pub fn new_y_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ONE, T::ZERO),
            radius,
            height,
        )
    }

    /// X軸に平行な円柱を作成（簡易コンストラクタ）
    pub fn new_x_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            height,
        )
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 底面の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 軸方向ベクトルを取得（正規化済み）
    pub fn axis(&self) -> Vector3D<T> {
        self.axis
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 高さを取得
    pub fn height(&self) -> T {
        self.height
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 円柱の体積を計算
    ///
    /// 体積 = π × r² × h
    pub fn volume(&self) -> T {
        T::PI * self.radius * self.radius * self.height
    }

    /// 円柱の表面積を計算
    ///
    /// 表面積 = 2π × r² + 2π × r × h (底面積 + 側面積)
    pub fn surface_area(&self) -> T {
        let base_area = T::PI * self.radius * self.radius;
        let side_area = T::from_f64(2.0) * T::PI * self.radius * self.height;
        T::from_f64(2.0) * base_area + side_area
    }

    /// 円柱の境界ボックスを計算
    pub fn bounding_box(&self) -> BBox3D<T> {
        // 軸に垂直な平面での最大範囲を計算
        let axis_x = self.axis.x().abs();
        let axis_y = self.axis.y().abs();
        let axis_z = self.axis.z().abs();

        // 軸方向の投影成分
        let height_x = self.height * axis_x;
        let height_y = self.height * axis_y;
        let height_z = self.height * axis_z;

        // 半径による拡張（軸に垂直な方向）
        let radius_x = self.radius * (T::ONE - axis_x * axis_x).sqrt();
        let radius_y = self.radius * (T::ONE - axis_y * axis_y).sqrt();
        let radius_z = self.radius * (T::ONE - axis_z * axis_z).sqrt();

        let min_x = (self.center.x() - radius_x).min(self.center.x() + height_x - radius_x);
        let max_x = (self.center.x() + radius_x).max(self.center.x() + height_x + radius_x);

        let min_y = (self.center.y() - radius_y).min(self.center.y() + height_y - radius_y);
        let max_y = (self.center.y() + radius_y).max(self.center.y() + height_y + radius_y);

        let min_z = (self.center.z() - radius_z).min(self.center.z() + height_z - radius_z);
        let max_z = (self.center.z() + radius_z).max(self.center.z() + height_z + radius_z);

        let min_point = Point3D::new(min_x, min_y, min_z);
        let max_point = Point3D::new(max_x, max_y, max_z);

        BBox3D::new(min_point, max_point)
    }

    // ========================================================================
    // Core Containment and Distance Methods
    // ========================================================================

    /// 点が円柱内部に含まれるかを判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        // 点から底面への投影を計算
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 軸方向の投影長
        let axis_projection = to_point.dot(&self.axis);

        // 高さ方向の範囲チェック
        if axis_projection < T::ZERO || axis_projection > self.height {
            return false;
        }

        // 軸からの距離を計算
        let axis_component = Vector3D::new(
            self.axis.x() * axis_projection,
            self.axis.y() * axis_projection,
            self.axis.z() * axis_projection,
        );
        let radial_component = to_point - axis_component;
        let radial_distance = radial_component.magnitude();

        radial_distance <= self.radius
    }

    /// 点から円柱表面までの距離を計算
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let axis_projection = to_point.dot(&self.axis);

        // 軸方向の距離
        let axis_distance = if axis_projection < T::ZERO {
            -axis_projection
        } else if axis_projection > self.height {
            axis_projection - self.height
        } else {
            T::ZERO
        };

        // 半径方向の距離
        let axis_component = Vector3D::new(
            self.axis.x() * axis_projection.max(T::ZERO).min(self.height),
            self.axis.y() * axis_projection.max(T::ZERO).min(self.height),
            self.axis.z() * axis_projection.max(T::ZERO).min(self.height),
        );
        let radial_component = to_point - axis_component;
        let radial_distance = radial_component.magnitude();
        let radial_excess = (radial_distance - self.radius).max(T::ZERO);

        // 軸方向と半径方向の距離を合成
        (axis_distance * axis_distance + radial_excess * radial_excess).sqrt()
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for Cylinder3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cylinder3D(center: {:?}, axis: {:?}, radius: {}, height: {})",
            self.center, self.axis, self.radius, self.height
        )
    }
}
