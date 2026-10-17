//! 3次元球ソリッドのCore実装
//!
//! STEP の SOLID_SPHERE と AXIS2_PLACEMENT_3D への対応

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D};
use geo_contracts::{
    default_distance_tolerance, PointClassification, Scalar, SphericalSolid3DConstructor,
    SphericalSolid3DContainment, SphericalSolid3DDistance, SphericalSolid3DProjection,
    SphericalSolid3DProperties,
};

/// STEP 準拠の 3 次元球ソリッド
///
/// 中心、参照軸、参照方向、半径を保持
/// 体積、表面積、包含判定を提供
#[derive(Debug, Clone, PartialEq)]
pub struct SphericalSolid3D<T: Scalar> {
    /// 球の中心点
    center: Point3D<T>,

    /// 参照軸方向
    axis: Direction3D<T>,

    /// 参照方向（axis と直交）
    ref_direction: Direction3D<T>,

    /// 球の半径
    radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for SphericalSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::SphericalSolid
    }
}

impl<T: Scalar> SphericalSolid3D<T> {
    /// 点から表面までの符号付き距離（内部は負）
    fn signed_distance_to_surface(&self, point: Point3D<T>) -> T {
        self.center.distance_to(point) - self.radius
    }
}

impl<T: Scalar> SphericalSolid3DConstructor<T> for SphericalSolid3D<T> {
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let ref_direction = Direction3D::from_vector(
            ref_direction.as_vector() - axis.as_vector() * ref_direction.dot(axis),
        )?;

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    fn new_standard(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            radius,
        )
    }
}

impl<T: Scalar> SphericalSolid3DProperties<T> for SphericalSolid3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }
}

impl<T: Scalar> SphericalSolid3DContainment<T> for SphericalSolid3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.center.distance_squared_to(point) <= self.radius * self.radius
    }

    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        if self.signed_distance_to_surface(point).abs() <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> SphericalSolid3DDistance<T> for SphericalSolid3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        self.signed_distance_to_surface(point).max(T::ZERO)
    }
}

impl<T: Scalar> SphericalSolid3DProjection<T> for SphericalSolid3D<T> {
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        let offset = point - self.center;
        let distance = offset.length();
        if distance <= self.radius {
            point
        } else {
            self.center + offset * (self.radius / distance)
        }
    }
}

impl<T: Scalar> std::fmt::Display for SphericalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SphericalSolid3D(center: ({}, {}, {}), axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {})",
            self.center.x(),
            self.center.y(),
            self.center.z(),
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
            self.radius
        )
    }
}
