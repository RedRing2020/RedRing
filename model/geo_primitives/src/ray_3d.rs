//! Ray3D - 3次元半無限直線のCore実装
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。
//! Core Traits実装（Constructor, Properties, Measure）も含む

use crate::{Direction3D, Point3D, Vector3D};
use analysis::linalg::{point3::Point3, vector::Vector3};
use geo_foundation::{
    core::ray_core_traits::{Ray3DConstructor, Ray3DMeasure, Ray3DProperties},
    Scalar,
};

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
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
        let direction_offset = self.direction * t;
        Point3D::new(
            self.origin.x() + direction_offset.x(),
            self.origin.y() + direction_offset.y(),
            self.origin.z() + direction_offset.z(),
        )
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

// ============================================================================
// Core Traits Implementation
// ============================================================================

/// Ray3DConstructor トレイト実装
impl<T: Scalar> Ray3DConstructor<T> for Ray3D<T> {
    fn new(origin: Point3<T>, direction: Vector3<T>) -> Option<Self>
    where
        Self: Sized,
    {
        let direction_vector = Vector3D::new(direction.x(), direction.y(), direction.z());
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::new(origin_point, direction_vector)
    }

    fn from_points(start: Point3<T>, through: Point3<T>) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point3D::new(start.x(), start.y(), start.z());
        let through_point = Point3D::new(through.x(), through.y(), through.z());
        Ray3D::from_points(start_point, through_point)
    }

    fn along_positive_x(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_x_axis(origin_point)
    }

    fn along_positive_y(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_y_axis(origin_point)
    }

    fn along_positive_z(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_z_axis(origin_point)
    }

    fn along_negative_x(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_x_direction = Vector3D::new(-T::ONE, T::ZERO, T::ZERO);
        Ray3D::new(origin_point, neg_x_direction).unwrap()
    }

    fn along_negative_y(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_y_direction = Vector3D::new(T::ZERO, -T::ONE, T::ZERO);
        Ray3D::new(origin_point, neg_y_direction).unwrap()
    }

    fn along_negative_z(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_z_direction = Vector3D::new(T::ZERO, T::ZERO, -T::ONE);
        Ray3D::new(origin_point, neg_z_direction).unwrap()
    }

    fn x_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_x_axis(Point3D::origin())
    }

    fn y_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_y_axis(Point3D::origin())
    }

    fn z_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_z_axis(Point3D::origin())
    }
}

/// Ray3DProperties トレイト実装
impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
    fn origin(&self) -> Point3<T> {
        let origin = self.origin();
        Point3::new(origin.x(), origin.y(), origin.z())
    }

    fn direction(&self) -> Vector3<T> {
        let direction = self.direction_vector();
        Vector3::new(direction.x(), direction.y(), direction.z())
    }

    fn origin_x(&self) -> T {
        self.origin().x()
    }

    fn origin_y(&self) -> T {
        self.origin().y()
    }

    fn origin_z(&self) -> T {
        self.origin().z()
    }

    fn direction_x(&self) -> T {
        self.direction_vector().x()
    }

    fn direction_y(&self) -> T {
        self.direction_vector().y()
    }

    fn direction_z(&self) -> T {
        self.direction_vector().z()
    }

    fn is_valid(&self) -> bool {
        // Ray3D::new がSomeを返した時点で有効性は保証されている
        true
    }
}

/// Ray3DMeasure トレイト実装
impl<T: Scalar> Ray3DMeasure<T> for Ray3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3<T> {
        let point = self.point_at_parameter(t);
        Point3::new(point.x(), point.y(), point.z())
    }

    fn closest_point(&self, point: &Point3<T>) -> Point3<T> {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        let t = self.parameter_for_point(&target_point);
        let clamped_t = if t < T::ZERO { T::ZERO } else { t };
        let closest = self.point_at_parameter(clamped_t);
        Point3::new(closest.x(), closest.y(), closest.z())
    }

    fn distance_to_point(&self, point: &Point3<T>) -> T {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        let to_point = target_point - self.origin();
        let projection_length = self.direction_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            self.origin().distance_to(&target_point)
        } else {
            let direction_offset = self.direction_vector() * projection_length;
            let projection = Point3D::new(
                self.origin().x() + direction_offset.x(),
                self.origin().y() + direction_offset.y(),
                self.origin().z() + direction_offset.z(),
            );
            target_point.distance_to(&projection)
        }
    }

    fn contains_point(&self, point: &Point3<T>) -> bool {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        use geo_foundation::tolerance_migration::DefaultTolerances;
        self.contains_point(&target_point, DefaultTolerances::distance::<T>())
    }

    fn parameter_for_point(&self, point: &Point3<T>) -> T {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        self.parameter_for_point(&target_point)
    }

    fn points_towards(&self, direction: &Vector3<T>) -> bool {
        let target_direction = Vector3D::new(direction.x(), direction.y(), direction.z());
        let dot = self.direction_vector().dot(&target_direction);
        dot > T::ZERO
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        let this_dir = self.direction_vector();
        let other_dir = other.direction_vector();

        let cross = this_dir.cross(&other_dir);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        cross.length() < DefaultTolerances::distance::<T>()
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction_vector().dot(&other.direction_vector());
        dot > T::ZERO
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction_vector().dot(&other.direction_vector());
        dot < T::ZERO
    }

    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        self.reverse_direction()
    }

    fn translate(&self, offset: Vector3<T>) -> Self
    where
        Self: Sized,
    {
        let offset_vector = Vector3D::new(offset.x(), offset.y(), offset.z());
        let new_origin = self.origin() + offset_vector;

        Ray3D::new(new_origin, self.direction_vector()).unwrap()
    }
}
