//! 3次元無限直線（InfiniteLine3D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine3Dの必須機能のみ
//! Foundation Pattern: Constructor/Properties を中心に capability trait を実装

mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, Angle, AngularRelation, InfiniteLine3DConstructor,
    InfiniteLine3DContainment, InfiniteLine3DDistance, InfiniteLine3DEvaluation,
    InfiniteLine3DProjection, InfiniteLine3DProperties, ParallelRelation, PerpendicularRelation,
    PrimitiveKind, PrimitiveMetadata, Scalar,
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

impl<T: Scalar> InfiniteLine3DConstructor<T> for InfiniteLine3D<T> {
    fn new(point: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        Direction3D::from_vector(direction).map(|direction| Self { point, direction })
    }

    fn from_two_points(p1: Point3D<T>, p2: Point3D<T>) -> Option<Self> {
        <Self as InfiniteLine3DConstructor<T>>::new(p1, Vector3D::from_points(p1, p2))
    }
}

impl<T: Scalar> InfiniteLine3DProperties<T> for InfiniteLine3D<T> {
    fn point(&self) -> Point3D<T> {
        self.point
    }

    fn direction(&self) -> Direction3D<T> {
        self.direction
    }

    fn dimension(&self) -> u32 {
        3
    }
}

impl<T: Scalar> InfiniteLine3DEvaluation<T> for InfiniteLine3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        Point3D::new(
            self.point.x() + t * self.direction.x(),
            self.point.y() + t * self.direction.y(),
            self.point.z() + t * self.direction.z(),
        )
    }

    fn parameter_for_point(&self, point: Point3D<T>) -> T {
        Vector3D::from_points(self.point, point).dot(*self.direction)
    }
}

impl<T: Scalar> InfiniteLine3DDistance<T> for InfiniteLine3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        point.distance_to(self.closest_point(point))
    }
}

impl<T: Scalar> InfiniteLine3DContainment<T> for InfiniteLine3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> InfiniteLine3DProjection<T> for InfiniteLine3D<T> {
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        self.point_at_parameter(self.parameter_for_point(point))
    }
}

impl<T: Scalar> ParallelRelation<Self> for InfiniteLine3D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction.is_parallel_to(other.direction)
    }
}

impl<T: Scalar> PerpendicularRelation<Self> for InfiniteLine3D<T> {
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(other.direction)
    }
}

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine3D<T> {
    /// 鋭角側の角度
    fn angle_to(&self, other: &Self) -> Angle<T> {
        let dot = self
            .direction
            .as_vector()
            .dot(other.direction.as_vector())
            .abs();
        Angle::from_radians(dot.max(T::ZERO).min(T::ONE).acos())
    }
}
