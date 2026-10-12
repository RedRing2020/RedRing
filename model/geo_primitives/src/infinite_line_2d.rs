//! 2次元無限直線（InfiniteLine2D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine2Dの必須機能のみ

mod extensions;
mod transform;

use crate::{Direction2D, Point2D, Vector2D};
use geo_contracts::{
    AngularRelation, ParallelRelation, PerpendicularRelation, PrimitiveKind, PrimitiveMetadata,
    SameLineRelation, Scalar,
};

/// 2次元無限直線（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine2D<T: Scalar> {
    point: Point2D<T>,         // 直線上の点
    direction: Direction2D<T>, // 正規化された方向ベクトル
}

impl<T: Scalar> PrimitiveMetadata for InfiniteLine2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }
}

impl<T: Scalar> InfiniteLine2D<T> {
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(point: Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        Direction2D::from_vector(direction).map(|dir| Self {
            point,
            direction: dir,
        })
    }

    /// 2点から無限直線を作成
    pub fn from_two_points(p1: Point2D<T>, p2: Point2D<T>) -> Option<Self> {
        let direction = Vector2D::from_points(p1, p2);
        Self::new(p1, direction)
    }

    /// 直線上の基準点を取得（線分から使うため pub(crate)）
    pub(crate) fn point_internal(&self) -> Point2D<T> {
        self.point
    }

    /// 直線の方向（正規化済み）を取得（線分から使うため pub(crate)）
    pub(crate) fn direction_internal(&self) -> Direction2D<T> {
        self.direction
    }

    /// 法線ベクトルを取得（右回り90度回転、内部用）
    fn normal_internal(&self) -> Direction2D<T> {
        Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid")
    }

    /// 指定パラメータでの点を取得（point + t * direction）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point + self.direction * t
    }

    /// 点から直線への最短距離を計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let to_point = Vector2D::from_points(self.point, *point);
        let normal = self.normal_internal();
        to_point.dot(&normal).abs()
    }

    /// 点が直線上にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 点を直線上に投影
    pub fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        let to_point = Vector2D::from_points(self.point, *point);
        let projection_length = to_point.dot(&self.direction);
        self.point_at_parameter(projection_length)
    }

    /// 点の直線に対するパラメータを取得
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        let to_point = Vector2D::from_points(self.point, *point);
        to_point.dot(&self.direction)
    }

    /// 境界ボックスを取得（起点を含む十分大きな範囲）
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        // 無限直線なので実用的な大きさの境界ボックスを生成
        let large_value = T::from_f64(1e6);
        let half_range = large_value;
        geo_core::Aabb2D::new(
            Point2D::new(self.point.x() - half_range, self.point.y() - half_range),
            Point2D::new(self.point.x() + half_range, self.point.y() + half_range),
        )
    }

    /// 接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        // 直線の接線ベクトルは方向ベクトルと同じ
        *self.direction
    }
}

use geo_contracts::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DDistance,
    InfiniteLine2DEvaluation, InfiniteLine2DProjection, InfiniteLine2DProperties,
    InfiniteLine2DTransform,
};

/// InfiniteLine2D Constructor Trait Implementation
impl<T: Scalar> InfiniteLine2DConstructor<T> for InfiniteLine2D<T> {
    fn new(point: (T, T), direction: (T, T)) -> Option<Self> {
        let point_2d = Point2D::new(point.0, point.1);
        let direction_vec = Vector2D::new(direction.0, direction.1);
        Self::new(point_2d, direction_vec)
    }

    fn from_two_points(p1: (T, T), p2: (T, T)) -> Option<Self> {
        let point1 = Point2D::new(p1.0, p1.1);
        let point2 = Point2D::new(p2.0, p2.1);
        Self::from_two_points(point1, point2)
    }

    fn horizontal(y: T) -> Self {
        let point = Point2D::new(T::ZERO, y);
        Self::new(point, Vector2D::unit_x()).unwrap()
    }
}

/// InfiniteLine2D Properties Trait Implementation
impl<T: Scalar> InfiniteLine2DProperties<T> for InfiniteLine2D<T> {
    fn point(&self) -> (T, T) {
        (self.point.x(), self.point.y())
    }

    fn direction(&self) -> (T, T) {
        (self.direction.x(), self.direction.y())
    }

    fn normal(&self) -> (T, T) {
        let n = Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid");
        (n.x(), n.y())
    }

    fn dimension(&self) -> u32 {
        2
    }

    fn angle(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }
}

impl<T: Scalar> InfiniteLine2DEvaluation<T> for InfiniteLine2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let p = self.point_at_parameter(t);
        (p.x(), p.y())
    }

    fn parameter_for_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.parameter_for_point(&p)
    }
}

impl<T: Scalar> InfiniteLine2DDistance<T> for InfiniteLine2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.distance_to_point(&p)
    }
}

impl<T: Scalar> InfiniteLine2DContainment<T> for InfiniteLine2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> InfiniteLine2DProjection<T> for InfiniteLine2D<T> {
    fn closest_point(&self, point: (T, T)) -> (T, T) {
        let p = Point2D::new(point.0, point.1);
        let projected = self.closest_point(&p);
        (projected.x(), projected.y())
    }
}

impl<T: Scalar> InfiniteLine2DTransform<T> for InfiniteLine2D<T> {
    fn reverse(&self) -> Self {
        Self::new(self.point, -(*self.direction)).unwrap()
    }
}

impl<T: Scalar> ParallelRelation<Self> for InfiniteLine2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
    }
}

impl<T: Scalar> PerpendicularRelation<Self> for InfiniteLine2D<T> {
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(&other.direction)
    }
}

impl<T: Scalar> SameLineRelation<Self> for InfiniteLine2D<T> {
    fn is_same_line(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
            && {
                use geo_contracts::default_distance_tolerance;
                self.contains_point(&other.point, default_distance_tolerance::<T>())
            }
    }
}

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine2D<T> {
    fn angle_to(&self, other: &Self) -> T {
        let dot = self.direction.dot(&other.direction);
        let clamped = dot.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }
}
