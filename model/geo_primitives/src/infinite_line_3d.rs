//! 3次元無限直線（InfiniteLine3D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine3Dの必須機能のみ
//! Foundation Pattern: Constructor/Properties を中心に capability trait を実装

use crate::{Direction3D, Plane3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, default_orthogonality_dot_error_tolerance,
    default_parallel_cross_error_tolerance, AngularRelation, BasicIntersection, ClosestPointPair,
    CrossDistance, InfiniteLine3DConstructor, InfiniteLine3DContainment, InfiniteLine3DDistance,
    InfiniteLine3DEvaluation, InfiniteLine3DProjection, InfiniteLine3DProperties,
    InfiniteLine3DTransform, IntersectsRelation, OnPlaneRelation, ParallelRelation,
    PerpendicularRelation, PrimitiveKind, PrimitiveMetadata, SameLineRelation, Scalar,
    SkewRelation,
};

type LinePointPair3D<T> = ((T, T, T), (T, T, T));

/// 3次元空間の無限直線（Core実装）
///
/// 点と方向ベクトルで定義される無限に延びる直線
/// Core機能：基本構築、アクセサ、基本幾何計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine3D<T: Scalar> {
    pub(crate) point: Point3D<T>,         // 直線上の任意の点
    pub(crate) direction: Direction3D<T>, // 方向ベクトル（正規化済み）
}

impl<T: Scalar> PrimitiveMetadata for InfiniteLine3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }
}

impl<T: Scalar> InfiniteLine3D<T> {
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

    /// 直線上の点を取得（内部用）
    pub(crate) fn point_internal(&self) -> Point3D<T> {
        self.point
    }

    /// 方向ベクトルを取得（正規化済み、内部用）
    pub(crate) fn direction_internal(&self) -> Direction3D<T> {
        self.direction
    }

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
        self.contains_point(point, default_distance_tolerance::<T>())
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
        self.direction.is_parallel_to(&other.direction)
    }

    /// 他の直線と垂直かを判定
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(&other.direction)
    }

    /// 他の直線とスキュー（ねじれ）関係にあるかを判定
    pub fn is_skew_to(&self, other: &Self) -> bool {
        !self.is_parallel_to(other) && !self.is_coplanar_with(other)
    }

    /// 他の直線との最近点対を取得
    pub fn closest_points(&self, other: &Self) -> Option<LinePointPair3D<T>> {
        if self.is_parallel_to(other) {
            return None;
        }

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
        if denom.abs() <= default_parallel_cross_error_tolerance::<T>() {
            return None;
        }

        let t1 = (b * e - c * d) / denom;
        let t2 = (a * e - b * d) / denom;
        let closest1 = self.point_at_parameter(t1);
        let closest2 = other.point_at_parameter(t2);
        Some((
            (closest1.x(), closest1.y(), closest1.z()),
            (closest2.x(), closest2.y(), closest2.z()),
        ))
    }

    /// 他の直線と同一直線かを判定
    pub fn is_same_line(&self, other: &Self) -> bool {
        self.is_parallel_to(other) && {
            let other_point = <Self as InfiniteLine3DProperties<T>>::point(other);
            <Self as InfiniteLine3DContainment<T>>::contains_point(self, other_point)
        }
    }

    /// 他の直線との角度を返す
    pub fn angle_to(&self, other: &Self) -> T {
        InfiniteLine3D::angle_with_line(self, other)
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
        scalar_triple.abs() <= default_distance_tolerance::<T>()
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

    fn from_xy_angle(angle: T) -> Self {
        let direction = Vector3D::new(angle.cos(), angle.sin(), T::ZERO);
        InfiniteLine3D::new(Point3D::origin(), direction).unwrap()
    }

    fn from_point_and_xy_angle(point: (T, T, T), angle: T) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        let direction = Vector3D::new(angle.cos(), angle.sin(), T::ZERO);
        InfiniteLine3D::new(p, direction).unwrap()
    }

    fn perpendicular_in_plane(
        point: (T, T, T),
        other: &Self,
        plane_normal: (T, T, T),
    ) -> Option<Self> {
        let p = Point3D::new(point.0, point.1, point.2);
        let other_dir = Vector3D::new(
            other.direction_internal().x(),
            other.direction_internal().y(),
            other.direction_internal().z(),
        );
        let normal = Vector3D::new(plane_normal.0, plane_normal.1, plane_normal.2);

        // 平面法線と直線方向の外積で垂直方向を計算
        let perp_dir = normal.cross(&other_dir);
        InfiniteLine3D::new(p, perp_dir)
    }
}

impl<T: Scalar> InfiniteLine3DProperties<T> for InfiniteLine3D<T> {
    fn point(&self) -> (T, T, T) {
        (self.point.x(), self.point.y(), self.point.z())
    }

    fn direction(&self) -> (T, T, T) {
        (self.direction.x(), self.direction.y(), self.direction.z())
    }

    fn is_x_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.y().abs() <= tolerance && self.direction.z().abs() <= tolerance
    }

    fn is_y_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.x().abs() <= tolerance && self.direction.z().abs() <= tolerance
    }

    fn is_z_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.x().abs() <= tolerance && self.direction.y().abs() <= tolerance
    }

    fn is_xy_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.z().abs() <= tolerance
    }

    fn is_xz_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.y().abs() <= tolerance
    }

    fn is_yz_parallel(&self) -> bool {
        let tolerance = default_parallel_cross_error_tolerance::<T>();
        self.direction.x().abs() <= tolerance
    }

    fn passes_through_origin(&self) -> bool {
        let origin = (T::ZERO, T::ZERO, T::ZERO);
        <Self as InfiniteLine3DContainment<T>>::contains_point(self, origin)
    }

    fn dimension(&self) -> u32 {
        3
    }

    fn xy_angle(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }

    fn is_axis_aligned(&self) -> bool {
        self.is_x_parallel() || self.is_y_parallel() || self.is_z_parallel()
    }
}

impl<T: Scalar> InfiniteLine3DEvaluation<T> for InfiniteLine3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let param_point = InfiniteLine3D::point_at_parameter(self, t);
        (param_point.x(), param_point.y(), param_point.z())
    }

    fn parameter_for_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        InfiniteLine3D::parameter_for_point(self, &p)
    }
}

impl<T: Scalar> InfiniteLine3DDistance<T> for InfiniteLine3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        InfiniteLine3D::distance_to_point(self, &p)
    }
}

impl<T: Scalar> InfiniteLine3DContainment<T> for InfiniteLine3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point(&p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> InfiniteLine3DProjection<T> for InfiniteLine3D<T> {
    fn project_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let projected = InfiniteLine3D::project_point(self, &p);
        (projected.x(), projected.y(), projected.z())
    }

    fn mirror_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let projected = self.project_point(&p);
        // 鏡面点 = 2 * 投影点 - 元の点
        let mirrored = projected + (projected - p);
        (mirrored.x(), mirrored.y(), mirrored.z())
    }
}

impl<T: Scalar> InfiniteLine3DTransform<T> for InfiniteLine3D<T> {
    fn reverse(&self) -> Self {
        let self_dir = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let self_point = <Self as InfiniteLine3DProperties<T>>::point(self);
        let reversed_dir = (-self_dir.0, -self_dir.1, -self_dir.2);
        <Self as InfiniteLine3DConstructor<T>>::new(self_point, reversed_dir).unwrap()
    }

    fn rotate_around_axis(
        &self,
        axis_point: (T, T, T),
        axis_direction: (T, T, T),
        angle: T,
    ) -> Option<Self> {
        use analysis::linalg::matrix::Matrix4x4;
        use analysis::linalg::vector::Vector3;

        let axis_vec = Vector3::new(axis_direction.0, axis_direction.1, axis_direction.2);
        let axis_normalized = axis_vec.normalize().ok()?;

        let rotation_matrix = Matrix4x4::rotation_axis(&axis_normalized, angle);

        // 軸上の点からの相対位置を計算して回転
        let axis_pt = Point3D::new(axis_point.0, axis_point.1, axis_point.2);
        let relative = Vector3D::from_points(&axis_pt, &self.point);
        let relative_analysis = Vector3::new(relative.x(), relative.y(), relative.z());
        let rotated_relative = rotation_matrix.transform_point_3d(&relative_analysis);

        let rotated_point = Point3D::new(
            axis_pt.x() + rotated_relative.x(),
            axis_pt.y() + rotated_relative.y(),
            axis_pt.z() + rotated_relative.z(),
        );

        // 方向ベクトルを回転
        let dir_analysis = Vector3::new(self.direction.x(), self.direction.y(), self.direction.z());
        let rotated_dir_analysis = rotation_matrix.transform_vector_3d(&dir_analysis);
        let rotated_dir = Vector3D::new(
            rotated_dir_analysis.x(),
            rotated_dir_analysis.y(),
            rotated_dir_analysis.z(),
        );

        InfiniteLine3D::new(rotated_point, rotated_dir)
    }
}

impl<T: Scalar> CrossDistance<T, Self> for InfiniteLine3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        InfiniteLine3D::distance_to_line(self, other)
    }
}

impl<T: Scalar> BasicIntersection<T, Self> for InfiniteLine3D<T> {
    type Point = (T, T, T);

    fn intersection_with(&self, other: &Self, _tolerance: T) -> Option<Self::Point> {
        // NOTE: この実装は既存挙動維持のため内部の既定トレランス判定を使用する。
        // `_tolerance` の設計見直しは別 Issue で扱う。
        if self.is_parallel_to(other) {
            return None;
        }
        if !self.is_coplanar_with(other) {
            return None;
        }
        let d1 = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let d2 = Vector3D::new(
            other.direction.x(),
            other.direction.y(),
            other.direction.z(),
        );
        let dp = Vector3D::from_points(&self.point, &other.point);
        let cross_d1_d2 = d1.cross(&d2);
        let cross_dp_d2 = dp.cross(&d2);
        let t = cross_dp_d2.dot(&cross_d1_d2) / cross_d1_d2.dot(&cross_d1_d2);
        let pt = self.point_at_parameter(t);
        Some((pt.x(), pt.y(), pt.z()))
    }
}

impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for InfiniteLine3D<T> {
    type Point = (T, T, T);

    fn intersection_with(&self, other: &Plane3D<T>, _tolerance: T) -> Option<Self::Point> {
        // NOTE: この実装は既存挙動維持のため内部の既定トレランス判定を使用する。
        // `_tolerance` の設計見直しは別 Issue で扱う。
        let plane_point = other.origin();
        let plane_normal = other.normal().as_vector();
        let line_dir = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());
        let denom = line_dir.dot(&plane_normal);
        if denom.abs() <= default_orthogonality_dot_error_tolerance::<T>() {
            return None;
        }
        let to_plane = Vector3D::from_points(&self.point, &plane_point);
        let t = to_plane.dot(&plane_normal) / denom;
        let pt = self.point_at_parameter(t);
        Some((pt.x(), pt.y(), pt.z()))
    }
}

impl<T: Scalar> ClosestPointPair<Self> for InfiniteLine3D<T> {
    type PointPair = ((T, T, T), (T, T, T));

    fn closest_points(&self, other: &Self) -> Option<Self::PointPair> {
        InfiniteLine3D::closest_points(self, other)
    }
}

impl<T: Scalar> ParallelRelation<Self> for InfiniteLine3D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_parallel_to(self, other)
    }
}

impl<T: Scalar> PerpendicularRelation<Self> for InfiniteLine3D<T> {
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_perpendicular_to(self, other)
    }
}

impl<T: Scalar> SameLineRelation<Self> for InfiniteLine3D<T> {
    fn is_same_line(&self, other: &Self) -> bool {
        InfiniteLine3D::is_same_line(self, other)
    }
}

impl<T: Scalar> IntersectsRelation<Self> for InfiniteLine3D<T> {
    fn intersects(&self, other: &Self) -> bool {
        InfiniteLine3D::distance_to_line(self, other) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> SkewRelation<Self> for InfiniteLine3D<T> {
    fn is_skew_to(&self, other: &Self) -> bool {
        InfiniteLine3D::is_skew_to(self, other)
    }
}

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine3D<T> {
    fn angle_to(&self, other: &Self) -> T {
        InfiniteLine3D::angle_to(self, other)
    }
}

impl<T: Scalar> OnPlaneRelation<Plane3D<T>> for InfiniteLine3D<T> {
    fn is_on_plane(&self, other: &Plane3D<T>) -> bool {
        let normal = other.normal().as_vector();
        let dir_vec = Vector3D::new(self.direction.x(), self.direction.y(), self.direction.z());

        if dir_vec.dot(&normal).abs() > default_orthogonality_dot_error_tolerance::<T>() {
            return false;
        }

        let plane_pt = other.origin();
        let to_line = Vector3D::from_points(&plane_pt, &self.point);
        to_line.dot(&normal).abs() <= default_distance_tolerance::<T>()
    }
}
