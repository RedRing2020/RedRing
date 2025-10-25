//! Ray3D - 3次元半無限直線のCore実装
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
pub struct Ray3D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point3D<T>,
    /// 方向ベクトル（正規化済み）
    direction: Vector3D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Ray3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

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

    /// 2点を通る Ray3D を作成
    ///
    /// # 引数
    /// * `start` - 起点
    /// * `through` - 通過する点
    ///
    /// # 戻り値
    /// 2点が同じ場合は None を返す
    pub fn from_points(start: Point3D<T>, through: Point3D<T>) -> Option<Self> {
        let direction = through - start;
        Self::new(start, direction)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 起点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// 方向ベクトルを取得
    pub fn direction(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.direction).expect("Ray direction should always be valid")
    }

    /// 内部方向ベクトルを取得（Vector3D型）
    pub fn direction_vector(&self) -> Vector3D<T> {
        self.direction
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// パラメータ t での点を計算
    ///
    /// # 引数
    /// * `t` - パラメータ（t ≥ 0）
    ///
    /// # 戻り値
    /// Ray上の点
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        self.origin + self.direction * t
    }

    /// 点がRay上にあるかを判定
    ///
    /// # 引数
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// Ray上にある場合は true
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let to_point = *point - self.origin;

        // 方向が同じかチェック
        let cross_product = self.direction.cross(&to_point);
        if cross_product.length() > tolerance {
            return false;
        }

        // パラメータが非負であるかチェック
        let t = self.direction.dot(&to_point);
        t >= -tolerance
    }

    /// 指定された点に対するパラメータを計算
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin;
        self.direction.dot(&to_point)
    }

    /// Ray の逆方向を作成
    pub fn reverse_direction(&self) -> Self {
        Self {
            origin: self.origin,
            direction: -self.direction,
        }
    }

    // ========================================================================
    // Core Axis-Aligned Ray Constructors
    // ========================================================================

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
