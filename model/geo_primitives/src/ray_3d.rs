//! Ray3D - 3次元半無限直線の実装
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。

#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, ParallelRelation, PrimitiveKind, PrimitiveMetadata,
    Ray3DConstructor, Ray3DContainment, Ray3DDistance, Ray3DEvaluation, Ray3DProjection,
    Ray3DProperties, Scalar, TolerantEq,
};

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point3D<T>,
    /// 方向（単位ベクトル）
    direction: Direction3D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Ray3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }
}

impl<T: Scalar> std::fmt::Display for Ray3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ray3D(origin: {:?}, direction: {:?})",
            self.origin,
            self.direction.as_vector()
        )
    }
}

impl<T: Scalar> TolerantEq<T> for Ray3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let origin_distance = self.origin.distance_to(other.origin);
        let direction_dot = self.direction.dot(other.direction).abs();
        let direction_similar = direction_dot >= T::from_f64(0.999);

        origin_distance <= tolerance && direction_similar
    }
}

impl<T: Scalar> Ray3DConstructor<T> for Ray3D<T> {
    fn new(origin: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        let direction = Direction3D::from_vector(direction)?;
        Some(Self { origin, direction })
    }

    fn from_points(start: Point3D<T>, through: Point3D<T>) -> Option<Self> {
        <Self as Ray3DConstructor<T>>::new(start, through - start)
    }
}

impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
    fn origin(&self) -> Point3D<T> {
        self.origin
    }

    fn direction(&self) -> Direction3D<T> {
        self.direction
    }

    fn is_valid(&self) -> bool {
        // 生成が Some を返した時点で有効性は保証されている
        true
    }
}

impl<T: Scalar> Ray3DEvaluation<T> for Ray3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let direction_offset = self.direction * t;
        Point3D::new(
            self.origin.x() + direction_offset.x(),
            self.origin.y() + direction_offset.y(),
            self.origin.z() + direction_offset.z(),
        )
    }

    fn parameter_for_point(&self, point: Point3D<T>) -> T {
        self.direction.as_vector().dot(point - self.origin)
    }
}

impl<T: Scalar> Ray3DProjection<T> for Ray3D<T> {
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        self.point_at_parameter(self.parameter_for_point(point).max(T::ZERO))
    }
}

impl<T: Scalar> Ray3DDistance<T> for Ray3D<T> {
    /// 起点の後方の点は、起点までの距離を返す
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        point.distance_to(self.closest_point(point))
    }
}

impl<T: Scalar> Ray3DContainment<T> for Ray3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> ParallelRelation<Self> for Ray3D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        let cross = self.direction.cross(*other.direction);
        cross.length() < default_distance_tolerance::<T>()
    }
}
