//! 3次元無限直線（InfiniteLine3D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine3Dの必須機能のみ

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の無限直線（Core実装）
///
/// 点と方向ベクトルで定義される無限に延びる直線
/// Core機能：基本構築、アクセサ、基本幾何計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine3D<T: Scalar> {
    point: Point3D<T>,         // 直線上の任意の点
    direction: Direction3D<T>, // 方向ベクトル（正規化済み）
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> InfiniteLine3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================
    /// 新しい無限直線を作成
    ///
    /// # 引数
    /// * `point` - 直線上の任意の点
    /// * `direction` - 方向ベクトル（自動的に正規化される）
    ///
    /// # 戻り値
    /// * `Some(InfiniteLine3D)` - 有効な直線が作成できた場合
    /// * `None` - 方向ベクトルがゼロベクトルの場合
    pub fn new(point: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        let direction_normalized = Direction3D::from_vector(direction)?;

        Some(Self {
            point,
            direction: direction_normalized,
        })
    }

    /// 2点から無限直線を作成
    pub fn from_two_points(p1: Point3D<T>, p2: Point3D<T>) -> Option<Self> {
        let direction = Vector3D::from_points(&p1, &p2);
        Self::new(p1, direction)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 直線上の点を取得
    pub fn point(&self) -> Point3D<T> {
        self.point
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Direction3D<T> {
        self.direction
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// パラメータtでの直線上の点を取得
    /// 点 = point + t * direction
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        Point3D::new(
            self.point.x() + t * self.direction.x(),
            self.point.y() + t * self.direction.y(),
            self.point.z() + t * self.direction.z(),
        )
    }

    /// 点を直線に投影
    pub fn project_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(&self.point, point);
        let t = to_point.dot(&self.direction);
        self.point_at_parameter(t)
    }

    /// 点から直線への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let projected = self.project_point(point);
        point.distance_to(&projected)
    }

    /// 点が直線上にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 点を直線に投影した時のパラメータtを取得
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.point, point);
        to_point.dot(&self.direction)
    }
}

// ============================================================================
// Foundation Pattern Core Traits Implementation
// ============================================================================

use geo_foundation::core::infinite_line_core_traits::{
    InfiniteLine3DConstructor, InfiniteLine3DMeasure, InfiniteLine3DProperties,
};

/// InfiniteLine3D Constructor Trait Implementation
impl<T: Scalar> InfiniteLine3DConstructor<T> for InfiniteLine3D<T> {
    fn new(point: (T, T, T), direction: (T, T, T)) -> Option<Self> {
        let point_3d = Point3D::new(point.0, point.1, point.2);
        let direction_vec = Vector3D::new(direction.0, direction.1, direction.2);
        Self::new(point_3d, direction_vec)
    }

    fn from_two_points(p1: (T, T, T), p2: (T, T, T)) -> Option<Self> {
        let point1 = Point3D::new(p1.0, p1.1, p1.2);
        let point2 = Point3D::new(p2.0, p2.1, p2.2);
        Self::from_two_points(point1, point2)
    }

    fn x_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        Self::new(p, Vector3D::unit_x()).unwrap()
    }

    fn y_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        Self::new(p, Vector3D::unit_y()).unwrap()
    }

    fn z_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        Self::new(p, Vector3D::unit_z()).unwrap()
    }

    fn x_axis() -> Self {
        Self::x_parallel((T::ZERO, T::ZERO, T::ZERO))
    }

    fn y_axis() -> Self {
        Self::y_parallel((T::ZERO, T::ZERO, T::ZERO))
    }

    fn z_axis() -> Self {
        Self::z_parallel((T::ZERO, T::ZERO, T::ZERO))
    }

    fn through_origin(direction: (T, T, T)) -> Option<Self> {
        let origin = Point3D::origin();
        let direction_vec = Vector3D::new(direction.0, direction.1, direction.2);
        Self::new(origin, direction_vec)
    }
}

/// InfiniteLine3D Properties Trait Implementation
impl<T: Scalar> InfiniteLine3DProperties<T> for InfiniteLine3D<T> {
    fn point(&self) -> (T, T, T) {
        let p = InfiniteLine3D::point(self);
        (p.x(), p.y(), p.z())
    }

    fn direction(&self) -> (T, T, T) {
        let d = InfiniteLine3D::direction(self);
        (d.x(), d.y(), d.z())
    }

    fn is_x_parallel(&self) -> bool {
        let d = InfiniteLine3D::direction(self);
        d.y().abs() <= T::EPSILON && d.z().abs() <= T::EPSILON
    }

    fn is_y_parallel(&self) -> bool {
        let d = InfiniteLine3D::direction(self);
        d.x().abs() <= T::EPSILON && d.z().abs() <= T::EPSILON
    }

    fn is_z_parallel(&self) -> bool {
        let d = InfiniteLine3D::direction(self);
        d.x().abs() <= T::EPSILON && d.y().abs() <= T::EPSILON
    }

    fn is_xy_parallel(&self) -> bool {
        InfiniteLine3D::direction(self).z().abs() <= T::EPSILON
    }

    fn is_xz_parallel(&self) -> bool {
        InfiniteLine3D::direction(self).y().abs() <= T::EPSILON
    }

    fn is_yz_parallel(&self) -> bool {
        InfiniteLine3D::direction(self).x().abs() <= T::EPSILON
    }

    fn passes_through_origin(&self) -> bool {
        use geo_foundation::tolerance_migration::DefaultTolerances;
        self.contains_point(&Point3D::origin(), DefaultTolerances::distance::<T>())
    }

    fn dimension(&self) -> u32 {
        3
    }
}

/// InfiniteLine3D Measure Trait Implementation
impl<T: Scalar> InfiniteLine3DMeasure<T> for InfiniteLine3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let p = InfiniteLine3D::point_at_parameter(self, t);
        (p.x(), p.y(), p.z())
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point(&p)
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        self.contains_point(&p, DefaultTolerances::distance::<T>())
    }

    fn project_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let projected = InfiniteLine3D::project_point(self, &p);
        (projected.x(), projected.y(), projected.z())
    }

    fn parameter_for_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        InfiniteLine3D::parameter_for_point(self, &p)
    }

    fn distance_to_line(&self, other: &Self) -> T {
        // 2つの3D直線間の最短距離
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let other_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(other);
        let dir1 = Vector3D::new(self_dir_tuple.0, self_dir_tuple.1, self_dir_tuple.2);
        let dir2 = Vector3D::new(other_dir_tuple.0, other_dir_tuple.1, other_dir_tuple.2);
        let cross = dir1.cross(&dir2);

        if cross.length() <= T::EPSILON {
            // 平行または同一直線の場合
            let other_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(other);
            return <Self as InfiniteLine3DMeasure<T>>::distance_to_point(self, other_point_tuple);
        }

        let self_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(self);
        let other_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(other);
        let p1 = Point3D::new(self_point_tuple.0, self_point_tuple.1, self_point_tuple.2);
        let p2 = Point3D::new(
            other_point_tuple.0,
            other_point_tuple.1,
            other_point_tuple.2,
        );
        let connecting = Vector3D::from_points(&p1, &p2);

        connecting.dot(&cross).abs() / cross.length()
    }

    fn closest_points(&self, other: &Self) -> Option<((T, T, T), (T, T, T))> {
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let other_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(other);
        let dir1 = Vector3D::new(self_dir_tuple.0, self_dir_tuple.1, self_dir_tuple.2);
        let dir2 = Vector3D::new(other_dir_tuple.0, other_dir_tuple.1, other_dir_tuple.2);
        let cross = dir1.cross(&dir2);

        if cross.length() <= T::EPSILON {
            // 平行線の場合
            return None;
        }

        let self_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(self);
        let other_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(other);
        let p1 = Point3D::new(self_point_tuple.0, self_point_tuple.1, self_point_tuple.2);
        let p2 = Point3D::new(
            other_point_tuple.0,
            other_point_tuple.1,
            other_point_tuple.2,
        );
        let w = Vector3D::from_points(&p2, &p1);

        let a = dir1.dot(&dir1);
        let b = dir1.dot(&dir2);
        let c = dir2.dot(&dir2);
        let d = dir1.dot(&w);
        let e = dir2.dot(&w);

        let denom = a * c - b * b;
        if denom.abs() <= T::EPSILON {
            return None;
        }

        let t1 = (b * e - c * d) / denom;
        let t2 = (a * e - b * d) / denom;

        let closest1 = <Self as InfiniteLine3DMeasure<T>>::point_at_parameter(self, t1);
        let closest2 = <Self as InfiniteLine3DMeasure<T>>::point_at_parameter(other, t2);

        Some((closest1, closest2))
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let other_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(other);
        let dir1 = Vector3D::new(self_dir_tuple.0, self_dir_tuple.1, self_dir_tuple.2);
        let dir2 = Vector3D::new(other_dir_tuple.0, other_dir_tuple.1, other_dir_tuple.2);
        let cross = dir1.cross(&dir2);
        cross.length() <= T::EPSILON
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let other_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(other);
        let dir1 = Vector3D::new(self_dir_tuple.0, self_dir_tuple.1, self_dir_tuple.2);
        let dir2 = Vector3D::new(other_dir_tuple.0, other_dir_tuple.1, other_dir_tuple.2);
        dir1.dot(&dir2).abs() <= T::EPSILON
    }

    fn is_same_line(&self, other: &Self) -> bool {
        // 平行かつ同じ点を含む場合
        <Self as InfiniteLine3DMeasure<T>>::is_parallel_to(self, other) && {
            let other_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(other);
            <Self as InfiniteLine3DMeasure<T>>::contains_point(self, other_point_tuple)
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.distance_to_line(other) <= T::EPSILON
    }

    fn is_skew_to(&self, other: &Self) -> bool {
        !self.is_parallel_to(other) && !self.intersects(other)
    }

    fn angle_to(&self, other: &Self) -> T {
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let other_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(other);
        let dir1 = Vector3D::new(self_dir_tuple.0, self_dir_tuple.1, self_dir_tuple.2);
        let dir2 = Vector3D::new(other_dir_tuple.0, other_dir_tuple.1, other_dir_tuple.2);
        let dot = dir1.dot(&dir2);
        let clamped = dot.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }

    fn reverse(&self) -> Self {
        let self_dir_tuple = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let self_point_tuple = <Self as InfiniteLine3DProperties<T>>::point(self);
        let reversed_dir_tuple = (-self_dir_tuple.0, -self_dir_tuple.1, -self_dir_tuple.2);
        <Self as InfiniteLine3DConstructor<T>>::new(self_point_tuple, reversed_dir_tuple).unwrap()
    }
}
