//! Ray2D - 2次元半無限直線の実装
//!
//! Ray2D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。

#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction2D, Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, default_kernel_numerical_zero_tolerance, ParallelRelation,
    PrimitiveKind, PrimitiveMetadata, Ray2DConstructor, Ray2DContainment, Ray2DDistance,
    Ray2DEvaluation, Ray2DProjection, Ray2DProperties, Scalar,
};

/// 2次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray2D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point2D<T>,
    /// 方向（単位ベクトル）
    direction: Direction2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Ray2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }
}

impl<T: Scalar> Ray2DConstructor<T> for Ray2D<T> {
    fn new(origin: Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        if direction.is_zero(default_kernel_numerical_zero_tolerance::<T>()) {
            return None;
        }
        let direction = Direction2D::from_vector(direction)?;
        Some(Self { origin, direction })
    }

    fn from_points(start: Point2D<T>, through: Point2D<T>) -> Option<Self> {
        <Self as Ray2DConstructor<T>>::new(start, through - start)
    }
}

impl<T: Scalar> Ray2DProperties<T> for Ray2D<T> {
    fn origin(&self) -> Point2D<T> {
        self.origin
    }

    fn direction(&self) -> Direction2D<T> {
        self.direction
    }

    fn is_valid(&self) -> bool {
        // 生成が Some を返した時点で有効性は保証されている
        true
    }
}

impl<T: Scalar> Ray2DEvaluation<T> for Ray2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.origin + self.direction * t
    }

    fn parameter_for_point(&self, point: Point2D<T>) -> T {
        (point - self.origin).dot(*self.direction)
    }
}

impl<T: Scalar> Ray2DProjection<T> for Ray2D<T> {
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T> {
        self.point_at_parameter(self.parameter_for_point(point).max(T::ZERO))
    }
}

impl<T: Scalar> Ray2DDistance<T> for Ray2D<T> {
    /// 起点の後方の点は、起点までの距離を返す
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        point.distance_to(self.closest_point(point))
    }
}

impl<T: Scalar> Ray2DContainment<T> for Ray2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> ParallelRelation<Self> for Ray2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        let cross = self
            .direction
            .as_vector()
            .cross(other.direction.as_vector());
        cross.abs() < default_distance_tolerance::<T>()
    }
}
