//! 3次元無限直線（InfiniteLine3D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine3Dの必須機能のみ
//! Foundation Pattern: Constructor/Properties を中心に capability trait を実装

mod extensions;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, AngularRelation, CrossDistance, InfiniteLine3DConstructor,
    InfiniteLine3DContainment, InfiniteLine3DDistance, InfiniteLine3DEvaluation,
    InfiniteLine3DProjection, InfiniteLine3DProperties, InfiniteLine3DTransform,
    IntersectsRelation, ParallelRelation, PerpendicularRelation, PrimitiveKind, PrimitiveMetadata,
    SameLineRelation, Scalar,
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

    /// 正規化済み方向ベクトルから無限直線を作成（再正規化なし）
    ///
    /// `Direction3D` は既に正規化済みであることが型で保証されるため、
    /// `new` と異なり sqrt を呼ばない。
    pub fn from_direction(point: Point3D<T>, direction: Direction3D<T>) -> Self {
        Self { point, direction }
    }

    /// 2点から無限直線を作成
    pub fn from_two_points(p1: Point3D<T>, p2: Point3D<T>) -> Option<Self> {
        let direction = Vector3D::from_points(&p1, &p2);
        Self::new(p1, direction)
    }

    /// 直線上の基準点を取得（線分から使うため pub(crate)）
    pub(crate) fn point_internal(&self) -> Point3D<T> {
        self.point
    }

    /// 直線の方向（正規化済み）を取得（線分から使うため pub(crate)）
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
    pub fn closest_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(&self.point, point);
        let t = to_point.dot(&self.direction);
        self.point_at_parameter(t)
    }

    /// 点から直線への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let projected = self.closest_point(point);
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

    /// 他の直線と平行かを判定
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction.is_parallel_to(&other.direction)
    }

    /// 他の直線と垂直かを判定
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(&other.direction)
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
}

impl<T: Scalar> InfiniteLine3DProperties<T> for InfiniteLine3D<T> {
    fn point(&self) -> (T, T, T) {
        (self.point.x(), self.point.y(), self.point.z())
    }

    fn direction(&self) -> (T, T, T) {
        (self.direction.x(), self.direction.y(), self.direction.z())
    }

    fn dimension(&self) -> u32 {
        3
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
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let projected = InfiniteLine3D::closest_point(self, &p);
        (projected.x(), projected.y(), projected.z())
    }
}

impl<T: Scalar> InfiniteLine3DTransform<T> for InfiniteLine3D<T> {
    fn reverse(&self) -> Self {
        let self_dir = <Self as InfiniteLine3DProperties<T>>::direction(self);
        let self_point = <Self as InfiniteLine3DProperties<T>>::point(self);
        let reversed_dir = (-self_dir.0, -self_dir.1, -self_dir.2);
        <Self as InfiniteLine3DConstructor<T>>::new(self_point, reversed_dir).unwrap()
    }
}

impl<T: Scalar> CrossDistance<T, Self> for InfiniteLine3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        InfiniteLine3D::distance_to_line(self, other)
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

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine3D<T> {
    fn angle_to(&self, other: &Self) -> T {
        InfiniteLine3D::angle_to(self, other)
    }
}
