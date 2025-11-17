//! 3次元無限直線（InfiniteLine3D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine3Dの必須機能のみ
//! Foundation Pattern: Constructor/Properties/Measure の3つのCore Traits実装

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::{
    InfiniteLine3DConstructor, InfiniteLine3DMeasure, InfiniteLine3DProperties, Scalar,
};

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

    /// 点が直線上にあるかを判定（デフォルトトレランス）
    pub fn contains_point_default(&self, point: &Point3D<T>) -> bool {
        self.contains_point(point, T::EPSILON)
    }

    /// 点に最も近い直線上の点を取得
    pub fn closest_point_to_point(&self, point: &Point3D<T>) -> Point3D<T> {
        self.project_point(point)
    }

    /// 点に対応するパラメータを取得
    pub fn parameter_at_point(&self, point: &Point3D<T>) -> Option<T> {
        if self.contains_point_default(point) {
            Some(self.parameter_for_point(point))
        } else {
            None
        }
    }

    /// 点を直線に投影した時のパラメータtを取得
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.point, point);
        to_point.dot(&self.direction)
    }

    /// 他の直線と平行かを判定
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        let cross = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z())
            .cross(&Vector3D::new(
                other.direction.x(),
                other.direction.y(),
                other.direction.z(),
            ));
        cross.length() <= T::EPSILON
    }

    /// 他の直線と垂直かを判定
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        let dot = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z()).dot(
            &Vector3D::new(
                other.direction.x(),
                other.direction.y(),
                other.direction.z(),
            ),
        );
        dot.abs() <= T::EPSILON
    }

    /// 他の直線とスキュー（ねじれ）関係にあるかを判定
    pub fn is_skew_to(&self, other: &Self) -> bool {
        !self.is_parallel_to(other) && !self.is_coplanar_with(other)
    }

    /// 他の直線と同一平面上にあるかを判定
    pub fn is_coplanar_with(&self, other: &Self) -> bool {
        let v1 = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let v2 = Vector3D::new(
            other.direction.x(),
            other.direction.y(),
            other.direction.z(),
        );
        let v3 = Vector3D::from_points(&self.point, &other.point);

        // スカラー三重積が0なら同一平面上
        let scalar_triple = v1.cross(&v2).dot(&v3);
        scalar_triple.abs() <= T::EPSILON
    }

    /// 他の直線との交点を計算
    pub fn intersection_with_line(&self, other: &Self) -> Option<Point3D<T>> {
        if self.is_parallel_to(other) {
            return None;
        }

        if !self.is_coplanar_with(other) {
            return None; // スキュー線は交差しない
        }

        // パラメトリック方程式を解く
        let p1 = self.point;
        let d1 = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let p2 = other.point;
        let d2 = Vector3D::new(
            other.direction.x(),
            other.direction.y(),
            other.direction.z(),
        );

        let dp = Vector3D::from_points(&p1, &p2);
        let cross_d1_d2 = d1.cross(&d2);
        let cross_dp_d2 = dp.cross(&d2);

        let t = cross_dp_d2.dot(&cross_d1_d2) / cross_d1_d2.dot(&cross_d1_d2);
        Some(self.point_at_parameter(t))
    }

    /// 他の直線との角度を計算（ラジアン）
    pub fn angle_with_line(&self, other: &Self) -> T {
        let d1 = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let d2 = Vector3D::new(
            other.direction.x(),
            other.direction.y(),
            other.direction.z(),
        );
        let dot = d1.dot(&d2).abs(); // 絶対値で鋭角を取得
        let clamped = dot.max(T::ZERO).min(T::ONE);
        clamped.acos()
    }

    /// 他の直線との最短距離を計算
    pub fn distance_to_line(&self, other: &Self) -> T {
        if self.is_parallel_to(other) {
            // 平行な場合、任意の点からの距離
            return self.distance_to_point(&other.point);
        }

        let d1 = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let d2 = Vector3D::new(
            other.direction.x(),
            other.direction.y(),
            other.direction.z(),
        );
        let dp = Vector3D::from_points(&self.point, &other.point);

        let cross = d1.cross(&d2);
        if cross.is_zero() {
            return T::ZERO; // 同一直線
        }

        dp.dot(&cross).abs() / cross.length()
    }

    /// 平面との交点を計算
    pub fn intersection_with_plane(
        &self,
        plane_point: &Point3D<T>,
        plane_normal: &Vector3D<T>,
    ) -> Option<Point3D<T>> {
        let line_dir = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let denom = line_dir.dot(plane_normal);

        if denom.abs() <= T::EPSILON {
            return None; // 直線が平面と平行
        }

        let to_plane = Vector3D::from_points(&self.point, plane_point);
        let t = to_plane.dot(plane_normal) / denom;
        Some(self.point_at_parameter(t))
    }

    /// 平面への投影を計算
    pub fn projection_on_plane(
        &self,
        plane_point: &Point3D<T>,
        plane_normal: &Vector3D<T>,
    ) -> Option<Self> {
        let line_dir = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());

        // 直線の方向を平面に投影
        let proj_dir = line_dir - (*plane_normal) * line_dir.dot(plane_normal);
        if proj_dir.is_zero() {
            return None; // 直線が平面に垂直
        }

        // 直線上の点を平面に投影
        let proj_point = {
            let to_plane = Vector3D::from_points(&self.point, plane_point);
            let dist_to_plane = to_plane.dot(plane_normal) / plane_normal.dot(plane_normal);
            Point3D::new(
                self.point.x() + dist_to_plane * plane_normal.x(),
                self.point.y() + dist_to_plane * plane_normal.y(),
                self.point.z() + dist_to_plane * plane_normal.z(),
            )
        };

        Self::new(proj_point, proj_dir)
    }

    /// 平面に対する反射を計算
    pub fn reflection_across_plane(
        &self,
        plane_point: &Point3D<T>,
        plane_normal: &Vector3D<T>,
    ) -> Option<Self> {
        let line_dir = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());

        // 方向ベクトルの反射
        let two = T::ONE + T::ONE;
        let refl_dir = line_dir - (*plane_normal) * (two * line_dir.dot(plane_normal));

        // 点の反射
        let to_plane = Vector3D::from_points(&self.point, plane_point);
        let dist_to_plane = to_plane.dot(plane_normal) / plane_normal.dot(plane_normal);
        let refl_point = Point3D::new(
            self.point.x() + two * dist_to_plane * plane_normal.x(),
            self.point.y() + two * dist_to_plane * plane_normal.y(),
            self.point.z() + two * dist_to_plane * plane_normal.z(),
        );

        Self::new(refl_point, refl_dir)
    }
}

// ============================================================================
// Foundation Pattern - Core Traits Implementation
// ============================================================================

// ============================================================================
// Constructor トレイト実装
// ============================================================================

impl<T: Scalar> InfiniteLine3DConstructor<T> for InfiniteLine3D<T> {
    fn new(point: (T, T, T), direction: (T, T, T)) -> Option<Self> {
        let p = Point3D::new(point.0, point.1, point.2);
        let d = Vector3D::new(direction.0, direction.1, direction.2);
        InfiniteLine3D::new(p, d)
    }

    fn from_two_points(point1: (T, T, T), point2: (T, T, T)) -> Option<Self> {
        let p1 = Point3D::new(point1.0, point1.1, point1.2);
        let p2 = Point3D::new(point2.0, point2.1, point2.2);
        InfiniteLine3D::from_two_points(p1, p2)
    }

    fn x_axis() -> Self {
        let origin = Point3D::origin();
        let direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        InfiniteLine3D::new(origin, direction).unwrap()
    }

    fn y_axis() -> Self {
        let origin = Point3D::origin();
        let direction = Vector3D::new(T::ZERO, T::ONE, T::ZERO);
        InfiniteLine3D::new(origin, direction).unwrap()
    }

    fn z_axis() -> Self {
        let origin = Point3D::origin();
        let direction = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        InfiniteLine3D::new(origin, direction).unwrap()
    }

    fn x_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        let direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        InfiniteLine3D::new(p, direction).unwrap()
    }

    fn y_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        let direction = Vector3D::new(T::ZERO, T::ONE, T::ZERO);
        InfiniteLine3D::new(p, direction).unwrap()
    }

    fn z_parallel(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        let direction = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        InfiniteLine3D::new(p, direction).unwrap()
    }

    fn through_origin(direction: (T, T, T)) -> Option<Self> {
        let origin = Point3D::origin();
        let dir = Vector3D::new(direction.0, direction.1, direction.2);
        InfiniteLine3D::new(origin, dir)
    }
}

// ============================================================================
// Properties トレイト実装
// ============================================================================

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
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.y().abs() <= tolerance && direction_obj.z().abs() <= tolerance
    }

    fn is_y_parallel(&self) -> bool {
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.x().abs() <= tolerance && direction_obj.z().abs() <= tolerance
    }

    fn is_z_parallel(&self) -> bool {
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.x().abs() <= tolerance && direction_obj.y().abs() <= tolerance
    }

    fn is_xy_parallel(&self) -> bool {
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.z().abs() <= tolerance
    }

    fn is_xz_parallel(&self) -> bool {
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.y().abs() <= tolerance
    }

    fn is_yz_parallel(&self) -> bool {
        let direction_obj = InfiniteLine3D::direction(self);
        let tolerance = T::EPSILON;
        direction_obj.x().abs() <= tolerance
    }

    fn passes_through_origin(&self) -> bool {
        let origin = (T::ZERO, T::ZERO, T::ZERO);
        <Self as InfiniteLine3DMeasure<T>>::contains_point(self, origin)
    }

    fn dimension(&self) -> u32 {
        3
    }
}

// ============================================================================
// Measure トレイト実装
// ============================================================================

impl<T: Scalar> InfiniteLine3DMeasure<T> for InfiniteLine3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let param_point = InfiniteLine3D::point_at_parameter(self, t);
        (param_point.x(), param_point.y(), param_point.z())
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        InfiniteLine3D::distance_to_point(self, &p)
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point(&p, T::EPSILON)
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
        InfiniteLine3D::distance_to_line(self, other)
    }

    fn closest_points(&self, other: &Self) -> Option<((T, T, T), (T, T, T))> {
        if self.is_parallel_to(other) {
            None
        } else {
            // 実装：最接近点の計算
            let self_dir = <Self as InfiniteLine3DProperties<T>>::direction(self);
            let other_dir = <Self as InfiniteLine3DProperties<T>>::direction(other);
            let self_point = <Self as InfiniteLine3DProperties<T>>::point(self);
            let other_point = <Self as InfiniteLine3DProperties<T>>::point(other);

            let d1 = Vector3D::new(self_dir.0, self_dir.1, self_dir.2);
            let d2 = Vector3D::new(other_dir.0, other_dir.1, other_dir.2);
            let p1 = Point3D::new(self_point.0, self_point.1, self_point.2);
            let p2 = Point3D::new(other_point.0, other_point.1, other_point.2);
            let w = Vector3D::from_points(&p2, &p1);

            let a = d1.dot(&d1);
            let b = d1.dot(&d2);
            let c = d2.dot(&d2);
            let d = d1.dot(&w);
            let e = d2.dot(&w);

            let denom = a * c - b * b;
            if denom.abs() <= T::EPSILON {
                None
            } else {
                let t1 = (b * e - c * d) / denom;
                let t2 = (a * e - b * d) / denom;

                let closest1 = <Self as InfiniteLine3DMeasure<T>>::point_at_parameter(self, t1);
                let closest2 = <Self as InfiniteLine3DMeasure<T>>::point_at_parameter(other, t2);
                Some((closest1, closest2))
            }
        }
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_parallel_to(self, other)
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_perpendicular_to(self, other)
    }

    fn is_same_line(&self, other: &Self) -> bool {
        self.is_parallel_to(other) && {
            let other_point = <Self as InfiniteLine3DProperties<T>>::point(other);
            <Self as InfiniteLine3DMeasure<T>>::contains_point(self, other_point)
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.distance_to_line(other) <= T::EPSILON
    }

    fn is_skew_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_skew_to(self, other)
    }

    fn angle_to(&self, other: &Self) -> T {
        InfiniteLine3D::angle_with_line(self, other)
    }

    fn reverse(&self) -> Self {
        let self_dir = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let self_point = <Self as InfiniteLine3DProperties<T>>::point(self);
        let reversed_dir = (-self_dir.0, -self_dir.1, -self_dir.2);
        <Self as InfiniteLine3DConstructor<T>>::new(self_point, reversed_dir).unwrap()
    }
}

// ============================================================================
