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

use crate::{Direction2D, InfiniteLine2D, Point2D, Vector2D};
use geo_contracts::{
    AngleBetween, BasicIntersection, DirectionalRelation, ParallelRelation, PointsTowards,
    Ray2DConstructor, Ray2DContainment, Ray2DDistance, Ray2DEvaluation, Ray2DProjection,
    Ray2DProperties, Ray2DTransform, Scalar,
};

/// 2次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray2D<T: Scalar> {
    /// 起点（t=0での点）
    pub(crate) origin: Point2D<T>,
    /// 方向ベクトル（正規化済み）
    pub(crate) direction: Vector2D<T>,
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

        let normalized_direction = direction.normalize();
        Some(Self {
            origin,
            direction: normalized_direction,
        })
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

    /// 起点を取得（内部用）
    pub(crate) fn origin_internal(&self) -> Point2D<T> {
        self.origin
    }

    /// 方向ベクトルを取得（正規化済み、内部用）
    pub(crate) fn direction_internal(&self) -> Direction2D<T> {
        Direction2D::from_vector(self.direction).unwrap()
    }

    /// 点が Ray 上にあるかを判定（tolerance付き）
    ///
    /// # 引数
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// 点が Ray 上にある場合は true
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 点から起点へのベクトル
        let to_point = *point - self.origin;

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

    /// Ray を InfiniteLine2D に変換
    pub fn to_infinite_line(&self) -> InfiniteLine2D<T> {
        InfiniteLine2D::new(self.origin, self.direction).unwrap()
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

    /// 指定方向を向いているかを判定
    pub fn points_towards_direction(&self, direction: (T, T)) -> bool {
        let target_direction = Vector2D::new(direction.0, direction.1);
        let self_direction =
            Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        self_direction.dot(&target_direction) > T::ZERO
    }

    /// 他の Ray との角度を返す
    pub fn angle_between(&self, other: &Self) -> T {
        let this_dir = Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let other_dir = Vector2D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
        );
        this_dir.dot(&other_dir).acos()
    }
}

// === Helper methods ===
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

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        // Ray のパラメータ範囲は [0, ∞)
        (T::ZERO, T::INFINITY)
    }

    /// 接線方向を取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        // Ray の接線方向は一定（方向ベクトル）
        self.direction
    }

    /// 方向を反転
    pub fn reverse_direction(&self) -> Self {
        Self::new(self.origin, -self.direction).unwrap()
    }

    /// 境界上判定（Rayでは点上判定と同じ）
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
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

// ============================================================================
// Core Traits Implementation
// ============================================================================

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

    fn along_positive_x(origin: (T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.0, origin.1);
        let x_direction = Vector2D::new(T::ONE, T::ZERO);
        Ray2D::new(origin_point, x_direction).unwrap()
    }

    fn along_positive_y(origin: (T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.0, origin.1);
        let y_direction = Vector2D::new(T::ZERO, T::ONE);
        Ray2D::new(origin_point, y_direction).unwrap()
    }

    fn along_negative_x(origin: (T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.0, origin.1);
        let neg_x_direction = Vector2D::new(-T::ONE, T::ZERO);
        Ray2D::new(origin_point, neg_x_direction).unwrap()
    }

    fn along_negative_y(origin: (T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.0, origin.1);
        let neg_y_direction = Vector2D::new(T::ZERO, -T::ONE);
        Ray2D::new(origin_point, neg_y_direction).unwrap()
    }

    fn x_axis() -> Self
    where
        Self: Sized,
    {
        Self::along_positive_x((T::ZERO, T::ZERO))
    }

    fn y_axis() -> Self
    where
        Self: Sized,
    {
        Self::along_positive_y((T::ZERO, T::ZERO))
    }

    // ========== Phase 2 実装 ==========

    fn from_angle(origin: (T, T), angle: T) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.0, origin.1);
        let direction = Vector2D::new(angle.cos(), angle.sin());
        Ray2D::new(origin_point, direction).unwrap()
    }

    fn horizontal_right() -> Self
    where
        Self: Sized,
    {
        Self::x_axis()
    }

    fn vertical_up() -> Self
    where
        Self: Sized,
    {
        Self::y_axis()
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

    fn origin_x(&self) -> T {
        self.origin.x()
    }

    fn origin_y(&self) -> T {
        self.origin.y()
    }

    fn direction_x(&self) -> T {
        self.direction.x()
    }

    fn direction_y(&self) -> T {
        self.direction.y()
    }

    fn is_valid(&self) -> bool {
        // Ray2D::new がSomeを返した時点で有効性は保証されている
        true
    }

    // ========== Phase 2 実装 ==========

    fn angle(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }

    fn is_horizontal(&self) -> bool {
        use geo_contracts::default_distance_tolerance;
        self.direction.y().abs() < default_distance_tolerance::<T>()
    }

    fn is_vertical(&self) -> bool {
        use geo_contracts::default_distance_tolerance;
        self.direction.x().abs() < default_distance_tolerance::<T>()
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

    fn point_at_distance(&self, distance: T) -> (T, T) {
        let point = self.point_at_parameter(distance);
        (point.x(), point.y())
    }
}

impl<T: Scalar> Ray2DProjection<T> for Ray2D<T> {
    fn closest_point(&self, point: (T, T)) -> (T, T) {
        let target_point = Point2D::new(point.0, point.1);
        let t = self.parameter_for_point(&target_point);
        let clamped_t = if t < T::ZERO { T::ZERO } else { t };
        let closest = self.point_at_parameter(clamped_t);
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
        let direction_vec =
            Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let reversed_direction = -direction_vec;
        Ray2D::new(self.origin_internal(), reversed_direction).unwrap()
    }

    fn translate(&self, offset: (T, T)) -> Self
    where
        Self: Sized,
    {
        let offset_vector = Vector2D::new(offset.0, offset.1);
        let new_origin = self.origin_internal() + offset_vector;
        let direction_vec =
            Vector2D::new(self.direction_internal().x(), self.direction_internal().y());

        Ray2D::new(new_origin, direction_vec).unwrap()
    }

    fn rotate_around_origin(&self, angle: T) -> Self
    where
        Self: Sized,
    {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let origin = self.origin_internal();
        let ox = origin.x();
        let oy = origin.y();
        let new_ox = ox * cos_a - oy * sin_a;
        let new_oy = ox * sin_a + oy * cos_a;
        let new_origin = Point2D::new(new_ox, new_oy);

        let dir = self.direction_internal();
        let dx = dir.x();
        let dy = dir.y();
        let new_dx = dx * cos_a - dy * sin_a;
        let new_dy = dx * sin_a + dy * cos_a;
        let new_direction = Vector2D::new(new_dx, new_dy);

        Ray2D::new(new_origin, new_direction).unwrap()
    }
}

impl<T: Scalar> BasicIntersection<T, Self> for Ray2D<T> {
    type Point = (T, T);

    fn intersection_with(&self, other: &Self, _tolerance: T) -> Option<Self::Point> {
        let this_dir = Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let other_dir = Vector2D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
        );

        let cross = this_dir.cross(&other_dir);
        use geo_contracts::default_distance_tolerance;
        if cross.abs() < default_distance_tolerance::<T>() {
            return None;
        }

        let diff = other.origin_internal() - self.origin_internal();
        let diff_vec = Vector2D::new(diff.x(), diff.y());
        let t = diff_vec.cross(&other_dir) / cross;

        if t < T::ZERO {
            return None;
        }

        let intersection = self.point_at_parameter(t);
        Some((intersection.x(), intersection.y()))
    }
}

impl<T: Scalar> PointsTowards<(T, T)> for Ray2D<T> {
    fn points_towards(&self, target: (T, T)) -> bool {
        Ray2D::points_towards_direction(self, target)
    }
}

impl<T: Scalar> ParallelRelation<Self> for Ray2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let other_dir = Vector2D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
        );
        let cross = this_dir.cross(&other_dir);
        cross.abs() < geo_contracts::default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> DirectionalRelation<Self> for Ray2D<T> {
    fn is_same_direction(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let other_dir = Vector2D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
        );
        let cross = this_dir.cross(&other_dir);
        if cross.abs() >= geo_contracts::default_distance_tolerance::<T>() {
            return false;
        }
        this_dir.dot(&other_dir) > T::ZERO
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        let other_dir = Vector2D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
        );
        let cross = this_dir.cross(&other_dir);
        if cross.abs() >= geo_contracts::default_distance_tolerance::<T>() {
            return false;
        }
        this_dir.dot(&other_dir) < T::ZERO
    }
}

impl<T: Scalar> AngleBetween<T, Self> for Ray2D<T> {
    fn angle_between(&self, other: &Self) -> T {
        Ray2D::angle_between(self, other)
    }
}
