//! Ray3D - 3次元半無限直線のCore実装
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。
//! Core Traits実装（Constructor, Properties, Measure）も含む

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{Ray3DConstructor, Ray3DMeasure, Ray3DProperties, Scalar};

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// 起点（t=0での点）
    pub(crate) origin: Point3D<T>,
    /// 方向ベクトル（正規化済み）
    pub(crate) direction: Vector3D<T>,
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

    /// 起点を取得（内部用）
    pub(crate) fn origin_internal(&self) -> Point3D<T> {
        self.origin
    }

    /// 方向ベクトルを取得（内部用）
    pub(crate) fn direction_internal(&self) -> Direction3D<T> {
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
    fn new(origin: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let direction_vector = Vector3D::new(direction.0, direction.1, direction.2);
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        Ray3D::new(origin_point, direction_vector)
    }

    fn from_points(start: (T, T, T), through: (T, T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let through_point = Point3D::new(through.0, through.1, through.2);
        Ray3D::from_points(start_point, through_point)
    }

    fn along_positive_x(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        Ray3D::along_x_axis(origin_point)
    }

    fn along_positive_y(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        Ray3D::along_y_axis(origin_point)
    }

    fn along_positive_z(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        Ray3D::along_z_axis(origin_point)
    }

    fn along_negative_x(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let neg_x_direction = Vector3D::new(-T::ONE, T::ZERO, T::ZERO);
        Ray3D::new(origin_point, neg_x_direction).unwrap()
    }

    fn along_negative_y(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let neg_y_direction = Vector3D::new(T::ZERO, -T::ONE, T::ZERO);
        Ray3D::new(origin_point, neg_y_direction).unwrap()
    }

    fn along_negative_z(origin: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
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

    // ========== Phase 2 実装 ==========

    fn from_spherical(origin: (T, T, T), azimuth: T, elevation: T) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let cos_elev = elevation.cos();
        let sin_elev = elevation.sin();
        let cos_azim = azimuth.cos();
        let sin_azim = azimuth.sin();

        let direction = Vector3D::new(cos_elev * cos_azim, cos_elev * sin_azim, sin_elev);
        Ray3D::new(origin_point, direction).unwrap()
    }

    fn xy_plane_angle(origin: (T, T, T), angle: T) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let direction = Vector3D::new(angle.cos(), angle.sin(), T::ZERO);
        Ray3D::new(origin_point, direction).unwrap()
    }

    fn xz_plane_angle(origin: (T, T, T), angle: T) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let direction = Vector3D::new(angle.cos(), T::ZERO, angle.sin());
        Ray3D::new(origin_point, direction).unwrap()
    }
}

/// Ray3DProperties トレイト実装
impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
    fn origin(&self) -> (T, T, T) {
        (self.origin.x(), self.origin.y(), self.origin.z())
    }

    fn direction(&self) -> (T, T, T) {
        (self.direction.x(), self.direction.y(), self.direction.z())
    }

    fn origin_x(&self) -> T {
        self.origin.x()
    }

    fn origin_y(&self) -> T {
        self.origin.y()
    }

    fn origin_z(&self) -> T {
        self.origin.z()
    }

    fn direction_x(&self) -> T {
        self.direction.x()
    }

    fn direction_y(&self) -> T {
        self.direction.y()
    }

    fn direction_z(&self) -> T {
        self.direction.z()
    }

    fn is_valid(&self) -> bool {
        // Ray3D::new がSomeを返した時点で有効性は保証されている
        true
    }

    // ========== Phase 2 実装 ==========

    fn azimuth(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }

    fn elevation(&self) -> T {
        let xy_length = (self.direction.x() * self.direction.x()
            + self.direction.y() * self.direction.y())
        .sqrt();
        self.direction.z().atan2(xy_length)
    }

    fn is_on_xy_plane(&self) -> bool {
        use geo_contracts::default_distance_tolerance;
        self.origin.z().abs() < default_distance_tolerance::<T>()
            && self.direction.z().abs() < default_distance_tolerance::<T>()
    }
}

/// Ray3DMeasure トレイト実装
impl<T: Scalar> Ray3DMeasure<T> for Ray3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let point = self.point_at_parameter(t);
        (point.x(), point.y(), point.z())
    }

    fn closest_point(&self, point: (T, T, T)) -> (T, T, T) {
        let target_point = Point3D::new(point.0, point.1, point.2);
        let t = self.parameter_for_point(&target_point);
        let clamped_t = if t < T::ZERO { T::ZERO } else { t };
        let closest = self.point_at_parameter(clamped_t);
        (closest.x(), closest.y(), closest.z())
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let target_point = Point3D::new(point.0, point.1, point.2);
        let to_point = target_point - self.origin;
        let projection_length = self.direction.dot(&to_point);

        if projection_length <= T::ZERO {
            self.origin.distance_to(&target_point)
        } else {
            let direction_offset = self.direction * projection_length;
            let projection = Point3D::new(
                self.origin.x() + direction_offset.x(),
                self.origin.y() + direction_offset.y(),
                self.origin.z() + direction_offset.z(),
            );
            target_point.distance_to(&projection)
        }
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let target_point = Point3D::new(point.0, point.1, point.2);
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&target_point, default_distance_tolerance::<T>())
    }

    fn parameter_for_point(&self, point: (T, T, T)) -> T {
        let target_point = Point3D::new(point.0, point.1, point.2);
        self.parameter_for_point(&target_point)
    }

    fn points_towards(&self, direction: (T, T, T)) -> bool {
        let target_direction = Vector3D::new(direction.0, direction.1, direction.2);
        let dot = self.direction.dot(&target_direction);
        dot > T::ZERO
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        let cross = self.direction.cross(&other.direction);
        use geo_contracts::default_distance_tolerance;
        cross.length() < default_distance_tolerance::<T>()
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction.dot(&other.direction);
        dot > T::ZERO
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction.dot(&other.direction);
        dot < T::ZERO
    }

    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        self.reverse_direction()
    }

    fn translate(&self, offset: (T, T, T)) -> Self
    where
        Self: Sized,
    {
        let offset_vector = Vector3D::new(offset.0, offset.1, offset.2);
        let new_origin = self.origin + offset_vector;

        Ray3D::new(new_origin, self.direction).unwrap()
    }

    // ========== Phase 2 実装 ==========

    fn distance_to_ray(&self, other: &Self) -> T {
        let w = self.origin - other.origin;
        let a = self.direction.dot(&self.direction);
        let b = self.direction.dot(&other.direction);
        let c = other.direction.dot(&other.direction);
        let d = self.direction.dot(&w);
        let e = other.direction.dot(&w);

        let denom = a * c - b * b;
        use geo_contracts::default_kernel_numerical_zero_tolerance;
        if denom.abs() < default_kernel_numerical_zero_tolerance::<T>() {
            // 平行: 片方の起点から他方への距離
            let other_origin = other.origin;
            return self.distance_to_point(&Point3D::new(
                other_origin.x(),
                other_origin.y(),
                other_origin.z(),
            ));
        }

        let sc = (b * e - c * d) / denom;
        let tc = (a * e - b * d) / denom;

        let sc_clamped = if sc < T::ZERO { T::ZERO } else { sc };
        let tc_clamped = if tc < T::ZERO { T::ZERO } else { tc };

        let p1 = self.point_at_parameter(sc_clamped);
        let p2 = other.point_at_parameter(tc_clamped);

        let diff = Vector3D::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        diff.length()
    }

    fn point_at_distance(&self, distance: T) -> (T, T, T) {
        // 方向ベクトルは正規化済みなので、パラメータ = 距離
        let point = self.point_at_parameter(distance);
        (point.x(), point.y(), point.z())
    }

    fn angle_between(&self, other: &Self) -> T {
        let dot = self.direction.dot(&other.direction);
        let clamped = if dot > T::ONE {
            T::ONE
        } else if dot < -T::ONE {
            -T::ONE
        } else {
            dot
        };
        clamped.acos()
    }

    fn rotate_around_axis(&self, axis: (T, T, T), angle: T) -> Option<Self>
    where
        Self: Sized,
    {
        use analysis::linalg::matrix::Matrix4x4;
        use analysis::linalg::vector::Vector3;

        // analysis::Vector3 に変換
        let axis_analysis = Vector3::new(axis.0, axis.1, axis.2);
        let norm = axis_analysis.norm();
        use geo_contracts::default_kernel_numerical_zero_tolerance;
        if norm < default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        let normalized_axis = axis_analysis.normalize().ok()?;
        let rotation_matrix = Matrix4x4::rotation_axis(&normalized_axis, angle);

        // 起点を回転
        let origin_analysis = Vector3::new(self.origin.x(), self.origin.y(), self.origin.z());
        let rotated_origin_analysis = rotation_matrix.transform_point_3d(&origin_analysis);
        let rotated_origin = Point3D::new(
            rotated_origin_analysis.x(),
            rotated_origin_analysis.y(),
            rotated_origin_analysis.z(),
        );

        // 方向ベクトルを回転
        let dir_analysis = Vector3::new(self.direction.x(), self.direction.y(), self.direction.z());
        let rotated_dir_analysis = rotation_matrix.transform_vector_3d(&dir_analysis);
        let rotated_dir = Vector3D::new(
            rotated_dir_analysis.x(),
            rotated_dir_analysis.y(),
            rotated_dir_analysis.z(),
        );

        Ray3D::new(rotated_origin, rotated_dir)
    }
}
