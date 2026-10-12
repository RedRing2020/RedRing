//! 2次元無限直線（InfiniteLine2D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine2Dの必須機能のみ

mod transform;

use crate::{Direction2D, Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, AngularRelation, InfiniteLine2DConstructor,
    InfiniteLine2DContainment, InfiniteLine2DDistance, InfiniteLine2DEvaluation,
    InfiniteLine2DProjection, InfiniteLine2DProperties, InfiniteLine2DTransform, ParallelRelation,
    PerpendicularRelation, PrimitiveKind, PrimitiveMetadata, Scalar,
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

impl<T: Scalar> InfiniteLine2DConstructor<T> for InfiniteLine2D<T> {
    fn new(point: &Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        Direction2D::from_vector(direction).map(|direction| Self {
            point: *point,
            direction,
        })
    }

    fn from_two_points(p1: &Point2D<T>, p2: &Point2D<T>) -> Option<Self> {
        <Self as InfiniteLine2DConstructor<T>>::new(p1, Vector2D::from_points(*p1, *p2))
    }

    fn horizontal(y: T) -> Self {
        Self {
            point: Point2D::new(T::ZERO, y),
            direction: Direction2D::positive_x(),
        }
    }
}

impl<T: Scalar> InfiniteLine2DProperties<T> for InfiniteLine2D<T> {
    fn point(&self) -> Point2D<T> {
        self.point
    }

    fn direction(&self) -> Direction2D<T> {
        self.direction
    }

    fn normal(&self) -> Direction2D<T> {
        Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid")
    }

    fn dimension(&self) -> u32 {
        2
    }
}

impl<T: Scalar> InfiniteLine2DEvaluation<T> for InfiniteLine2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point + self.direction * t
    }

    fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        Vector2D::from_points(self.point, *point).dot(&self.direction)
    }
}

impl<T: Scalar> InfiniteLine2DDistance<T> for InfiniteLine2D<T> {
    fn distance_to_point(&self, point: &Point2D<T>) -> T {
        Vector2D::from_points(self.point, *point)
            .dot(&self.normal())
            .abs()
    }
}

impl<T: Scalar> InfiniteLine2DContainment<T> for InfiniteLine2D<T> {
    fn contains_point(&self, point: &Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> InfiniteLine2DProjection<T> for InfiniteLine2D<T> {
    fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        self.point_at_parameter(self.parameter_for_point(point))
    }
}

impl<T: Scalar> InfiniteLine2DTransform<T> for InfiniteLine2D<T> {
    fn reverse(&self) -> Self {
        Self {
            point: self.point,
            direction: -self.direction,
        }
    }
}

impl<T: Scalar> ParallelRelation<Self> for InfiniteLine2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction.is_parallel_to(&other.direction)
    }
}

impl<T: Scalar> PerpendicularRelation<Self> for InfiniteLine2D<T> {
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(&other.direction)
    }
}

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine2D<T> {
    fn angle_to(&self, other: &Self) -> T {
        let dot = self.direction.dot(&other.direction);
        let clamped = dot.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }
}
