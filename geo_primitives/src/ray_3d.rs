//! Ray3D - 3次元半無限直線の実装（Core Foundation）
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。
//!
//! # Core Foundation パターン
//!
//! ## Core Foundation（120-150行）
//! - 基本プロパティ（origin, direction）
//! - Core 作成メソッド（new, from_points）
//! - 基本的な幾何操作（point_at_parameter, contains_point）
//! - InfiniteLine3D への変換
//! - 基本トレイト実装（CoreFoundation, BasicParametric, BasicDirectional, BasicContainment）

use crate::{Direction3D, InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point3D<T>,
    /// 方向ベクトル（正規化済み）
    direction: Vector3D<T>,
}

impl<T: Scalar> Ray3D<T> {
    /// 起点と方向ベクトルから Ray3D を作成
    ///
    /// # 引数
    /// * `origin` - 起点
    /// * `direction` - 方向ベクトル（自動的に正規化される）
    ///
    /// # 戻り値
    /// 方向ベクトルがゼロベクトルの場合は None を返す
    pub fn new(origin: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        if direction.is_zero() {
            return None;
        }

        let normalized_direction = direction.normalize();
        Some(Self {
            origin,
            direction: normalized_direction,
        })
    }

    /// 2点から Ray3D を作成
    ///
    /// # 引数
    /// * `start` - 起点
    /// * `through` - Ray が通る点（start と異なる必要がある）
    ///
    /// # 戻り値
    /// 2点が同一の場合は None を返す
    pub fn from_points(start: Point3D<T>, through: Point3D<T>) -> Option<Self> {
        let direction_vector = Vector3D::from_points(&start, &through);
        Self::new(start, direction_vector)
    }

    /// 起点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.direction).unwrap()
    }

    /// 点が Ray 上にあるかを判定（tolerance付き）
    ///
    /// # 引数
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// 点が Ray 上にある場合は true
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // 点から起点へのベクトル
        let to_point = Vector3D::from_points(&self.origin, point);

        // 方向ベクトルとの内積でパラメータ t を計算
        let t = to_point.dot(&self.direction);

        // t >= 0 かつ点が直線上にある
        if t < T::ZERO {
            return false;
        }

        // 直線上の点との距離をチェック
        let projected_point = self.origin + self.direction * t;
        let distance = point.distance_to(&projected_point);
        distance <= tolerance
    }

    /// Ray を InfiniteLine3D に変換
    pub fn to_infinite_line(&self) -> InfiniteLine3D<T> {
        InfiniteLine3D::new(self.origin, self.direction).unwrap()
    }

    /// 点に対するパラメータ t を取得
    ///
    /// # 引数
    /// * `point` - パラメータを求める点
    ///
    /// # 戻り値
    /// 点が Ray の延長線上にある場合のパラメータ（負の値も含む）
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.origin, point);
        to_point.dot(&self.direction)
    }
}

// === Helper methods ===
impl<T: Scalar> Ray3D<T> {
    /// 境界ボックスを取得（起点のみ）
    pub fn bounding_box(&self) -> crate::BBox3D<T> {
        // Ray は無限なので、境界ボックスは起点のみで構成
        // 実際の用途では適切な範囲を指定する必要がある
        crate::BBox3D::<T>::from_point(self.origin)
    }

    /// パラメータ位置の点を取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        // Ray では t >= 0 のみ有効だが、計算上は制限なし
        self.origin + self.direction * t
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        // Ray のパラメータ範囲は [0, ∞)
        (T::ZERO, T::INFINITY)
    }

    /// 接線方向を取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector3D<T> {
        // Ray の接線方向は一定（方向ベクトル）
        self.direction
    }

    /// 方向を反転
    pub fn reverse_direction(&self) -> Self {
        Self::new(self.origin, -self.direction).unwrap()
    }

    /// 境界上判定（Rayでは点上判定と同じ）
    pub fn on_boundary(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    /// 点からの距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let t = self.parameter_for_point(point);

        if t >= T::ZERO {
            // 点が Ray の有効範囲内
            let projected_point = self.point_at_parameter(t);
            point.distance_to(&projected_point)
        } else {
            // 点が Ray の起点より後ろ側
            point.distance_to(&self.origin)
        }
    }

    /// 指定方向に向いているかを判定
    pub fn points_towards(&self, target: &Point3D<T>) -> bool {
        let to_target = Vector3D::from_points(&self.origin, target);
        self.direction.dot(&to_target) > T::ZERO
    }

    /// 点が Ray の正の方向にあるかを判定
    pub fn is_point_ahead(&self, point: &Point3D<T>) -> bool {
        self.parameter_for_point(point) >= T::ZERO
    }

    /// 点が Ray の開始点より後ろにあるかを判定
    pub fn is_point_behind(&self, point: &Point3D<T>) -> bool {
        self.parameter_for_point(point) < T::ZERO
    }

    /// 射線上で点に最も近い点を取得
    pub fn closest_point_on_ray(&self, point: &Point3D<T>) -> Point3D<T> {
        let t = self.parameter_for_point(point);

        if t >= T::ZERO {
            // 点が Ray の有効範囲内
            self.point_at_parameter(t)
        } else {
            // 点が Ray の起点より後ろ側の場合は起点が最も近い
            self.origin
        }
    }

    /// 射線上で点に最も近い点のパラメータを取得
    pub fn closest_parameter(&self, point: &Point3D<T>) -> T {
        let t = self.parameter_for_point(point);
        t.max(T::ZERO) // t < 0 の場合は 0 (起点) を返す
    }

    /// X軸に平行な Ray を作成
    pub fn along_x_axis(origin: Point3D<T>) -> Self {
        Self::new(origin, Vector3D::unit_x()).unwrap()
    }

    /// Y軸に平行な Ray を作成
    pub fn along_y_axis(origin: Point3D<T>) -> Self {
        Self::new(origin, Vector3D::unit_y()).unwrap()
    }

    /// Z軸に平行な Ray を作成
    pub fn along_z_axis(origin: Point3D<T>) -> Self {
        Self::new(origin, Vector3D::unit_z()).unwrap()
    }
}

// トレイト実装
// TODO: CoreFoundation実装後に有効化
/*
impl<T: Scalar> BasicDirectional<T> for Ray3D<T> {
    type Direction = Direction3D<T>;

    fn direction(&self) -> Self::Direction {
        self.direction()
    }

    fn reverse_direction(&self) -> Self {
        self.reverse_direction()
    }
}
*/
