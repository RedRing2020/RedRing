//! Ray2D - 2次元半無限直線の実装（Core Foundation）
//!
//! Ray2D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。
//!
//! # Core Foundation パターン
//!
//! ## Core Foundation（120-150行）
//! - 基本プロパティ（origin, direction）
//! - Core 作成メソッド（new, from_points）
//! - 基本的な幾何操作（point_at_parameter, contains_point）
//! - InfiniteLine2D への変換
//! - 基本トレイト実装（CoreFoundation, BasicParametric, BasicDirectional, BasicContainment）
//! - Core Traits実装（Constructor, Properties, Measure）

mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction2D, Point2D, Vector2D};
use geo_contracts::{
    AngleBetween, ParallelRelation, PrimitiveKind, PrimitiveMetadata, Ray2DConstructor,
    Ray2DContainment, Ray2DDistance, Ray2DEvaluation, Ray2DProjection, Ray2DProperties,
    Ray2DTransform, Scalar,
};

/// 2次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray2D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point2D<T>,
    /// 方向（単位ベクトル）
    direction: Direction2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Ray2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }
}

impl<T: Scalar> Ray2D<T> {
    /// 起点と方向ベクトルから Ray2D を作成
    ///
    /// # 引数
    /// * `origin` - 起点
    /// * `direction` - 方向ベクトル（自動的に正規化される）
    ///
    /// # 戻り値
    /// 方向ベクトルがゼロベクトルの場合は None を返す
    pub fn new(origin: Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        if direction.is_zero(geo_contracts::default_kernel_numerical_zero_tolerance::<T>()) {
            return None;
        }

        let direction = Direction2D::from_vector(direction)?;
        Some(Self { origin, direction })
    }

    /// 2点から Ray2D を作成
    ///
    /// # 引数
    /// * `start` - 起点
    /// * `through` - Ray が通る点（start と異なる必要がある）
    ///
    /// # 戻り値
    /// 2点が同一の場合は None を返す
    pub fn from_points(start: Point2D<T>, through: Point2D<T>) -> Option<Self> {
        let direction_vector = through - start;
        Self::new(start, direction_vector)
    }

    /// 点が Ray 上にあるかを判定（tolerance付き）
    ///
    /// # 引数
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// Ray までの距離が `tolerance` 以内なら true（起点の後方も起点までの距離で判定する）
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 点に対するパラメータ t を取得
    ///
    /// # 引数
    /// * `point` - パラメータを求める点
    ///
    /// # 戻り値
    /// 点が Ray の延長線上にある場合のパラメータ（負の値も含む）
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        let to_point = *point - self.origin;
        to_point.dot(&self.direction)
    }

    /// Ray 上で点に最も近い点のパラメータ（起点からの距離。0 以上に制限する）を取得
    pub fn closest_parameter(&self, point: &Point2D<T>) -> T {
        self.parameter_for_point(point).max(T::ZERO)
    }

    /// Ray 上で点に最も近い点を取得
    pub fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        self.point_at_parameter(self.closest_parameter(point))
    }

    /// 他の Ray との角度を返す
    pub fn angle_between(&self, other: &Self) -> T {
        let this_dir = Vector2D::new(self.direction.x(), self.direction.y());
        let other_dir = Vector2D::new(other.direction.x(), other.direction.y());
        this_dir.dot(&other_dir).acos()
    }
}

impl<T: Scalar> Ray2D<T> {
    /// 境界ボックスを取得（起点のみ）
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        // Ray は無限なので、境界ボックスは起点のみで構成
        // 実際の用途では適切な範囲を指定する必要がある
        geo_core::Aabb2D::new(
            Point2D::new(self.origin.x(), self.origin.y()),
            Point2D::new(self.origin.x(), self.origin.y()),
        )
    }

    /// パラメータ位置の点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        // Ray では t >= 0 のみ有効だが、計算上は制限なし
        self.origin + self.direction * t
    }

    /// 接線方向を取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        // Ray の接線方向は一定（方向ベクトル）
        self.direction.as_vector()
    }

    /// 点からの距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let t = self.parameter_for_point(point);

        if t >= T::ZERO {
            // 点が Ray の有効範囲内
            let projected_point = self.origin + self.direction * t;
            point.distance_to(&projected_point)
        } else {
            // 点が Ray の起点より後ろ側
            point.distance_to(&self.origin)
        }
    }
}

/// Ray2DConstructor トレイト実装
impl<T: Scalar> Ray2DConstructor<T> for Ray2D<T> {
    fn new(origin: (T, T), direction: (T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let direction_vector = Vector2D::new(direction.0, direction.1);
        let origin_point = Point2D::new(origin.0, origin.1);
        Ray2D::new(origin_point, direction_vector)
    }

    fn from_points(start: (T, T), through: (T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point2D::new(start.0, start.1);
        let through_point = Point2D::new(through.0, through.1);
        Ray2D::from_points(start_point, through_point)
    }
}

/// Ray2DProperties トレイト実装
impl<T: Scalar> Ray2DProperties<T> for Ray2D<T> {
    fn origin(&self) -> (T, T) {
        (self.origin.x(), self.origin.y())
    }

    fn direction(&self) -> (T, T) {
        (self.direction.x(), self.direction.y())
    }

    fn is_valid(&self) -> bool {
        // Ray2D::new がSomeを返した時点で有効性は保証されている
        true
    }

    fn angle(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }
}

impl<T: Scalar> Ray2DEvaluation<T> for Ray2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let point = self.point_at_parameter(t);
        (point.x(), point.y())
    }

    fn parameter_for_point(&self, point: (T, T)) -> T {
        let target_point = Point2D::new(point.0, point.1);
        self.parameter_for_point(&target_point)
    }
}

impl<T: Scalar> Ray2DProjection<T> for Ray2D<T> {
    fn closest_point(&self, point: (T, T)) -> (T, T) {
        let closest = Ray2D::closest_point(self, &Point2D::new(point.0, point.1));
        (closest.x(), closest.y())
    }
}

impl<T: Scalar> Ray2DDistance<T> for Ray2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let target_point = Point2D::new(point.0, point.1);
        self.distance_to_point(&target_point)
    }
}

impl<T: Scalar> Ray2DContainment<T> for Ray2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let target_point = Point2D::new(point.0, point.1);
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&target_point, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> Ray2DTransform<T> for Ray2D<T> {
    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        let direction_vec = Vector2D::new(self.direction.x(), self.direction.y());
        let reversed_direction = -direction_vec;
        Ray2D::new(self.origin, reversed_direction).unwrap()
    }
}

impl<T: Scalar> ParallelRelation<Self> for Ray2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction.x(), self.direction.y());
        let other_dir = Vector2D::new(other.direction.x(), other.direction.y());
        let cross = this_dir.cross(&other_dir);
        cross.abs() < geo_contracts::default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> AngleBetween<T, Self> for Ray2D<T> {
    fn angle_between(&self, other: &Self) -> T {
        Ray2D::angle_between(self, other)
    }
}
