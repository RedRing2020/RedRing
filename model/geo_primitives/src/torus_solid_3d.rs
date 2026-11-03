mod bounds;
#[cfg(test)]
mod tests;
mod transform;

// torus_solid_3d.rs
// STEP AP214 準拠のトーラス固体実装
//
// トーラス固体は大半径（major_radius）と小半径（minor_radius）を持つ回転固体です。
//
// STEP エンティティ: TORUS + AXIS2_PLACEMENT_3D
// 固体としての体積と表面を持ちます。

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, PointClassification, Scalar, TorusSolid3DConstructor,
    TorusSolid3DContainment, TorusSolid3DDistance, TorusSolid3DProperties,
};

/// STEP AP214 準拠のトーラス固体
///
/// 大半径（中心軸から断面の円の中心までの距離）と
/// 小半径（断面の円の半径）により定義される3次元固体です。
#[derive(Debug, Clone, PartialEq)]
pub struct TorusSolid3D<T: Scalar> {
    /// 原点（トーラスの中心点）
    origin: Point3D<T>,
    /// Z軸方向（トーラスの軸方向）
    z_axis: Direction3D<T>,
    /// X軸方向（大半径の基準方向。z_axis と直交）
    x_axis: Direction3D<T>,
    /// 大半径（中心軸から断面の円の中心までの距離）
    major_radius: T,
    /// 小半径（断面の円の半径）
    minor_radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for TorusSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::TorusSolid
    }
}

impl<T: Scalar> TorusSolid3D<T> {
    /// 点から、断面の円の中心が描く円までの距離
    fn tube_center_distance(&self, point: Point3D<T>) -> T {
        let local = point - self.origin;
        let axial = local.dot(self.z_axis.as_vector());
        let radial = (local - self.z_axis.as_vector() * axial).length();
        Vector3D::new(radial - self.major_radius, axial, T::ZERO).length()
    }
}

impl<T: Scalar> TorusSolid3DConstructor<T> for TorusSolid3D<T> {
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        // 大半径が小半径以下だと、断面が中心軸で重なり立体にならない
        if minor_radius <= T::ZERO || major_radius <= minor_radius {
            return None;
        }

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let x_axis = Direction3D::from_vector(
            ref_direction.as_vector() - axis.as_vector() * ref_direction.dot(axis),
        )?;

        Some(Self {
            origin: center,
            z_axis: axis,
            x_axis,
            major_radius,
            minor_radius,
        })
    }

    fn new_standard(center: Point3D<T>, major_radius: T, minor_radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            major_radius,
            minor_radius,
        )
    }
}

impl<T: Scalar> TorusSolid3DProperties<T> for TorusSolid3D<T> {
    fn center(&self) -> Point3D<T> {
        self.origin
    }

    fn axis(&self) -> Direction3D<T> {
        self.z_axis
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.x_axis
    }

    fn major_radius(&self) -> T {
        self.major_radius
    }

    fn minor_radius(&self) -> T {
        self.minor_radius
    }
}

impl<T: Scalar> TorusSolid3DContainment<T> for TorusSolid3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.tube_center_distance(point) <= self.minor_radius
    }

    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        let boundary_distance = (self.tube_center_distance(point) - self.minor_radius).abs();
        if boundary_distance <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> TorusSolid3DDistance<T> for TorusSolid3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        (self.tube_center_distance(point) - self.minor_radius).max(T::ZERO)
    }
}
