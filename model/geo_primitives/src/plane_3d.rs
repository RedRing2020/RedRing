//! Plane3D 実装
//!
//! 座標系付き平面（STEP AP214 の AXIS2_PLACEMENT_3D + PLANE に対応）

#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, Plane3DConstructor, Plane3DContainment, Plane3DDistance,
    Plane3DProjection, Plane3DProperties, PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq,
};

/// 3次元平面（座標系付き）
///
/// ## 座標系定義
/// - origin: 平面原点（STEP: location）
/// - normal: Z軸方向（STEP: axis）- 法線ベクトル
/// - u_axis: X軸方向（STEP: ref_direction）- 第一軸
/// - v_axis: Y軸方向（STEP: derived）- normal × u_axis で自動計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane3D<T: Scalar> {
    /// 平面原点（STEP: location）
    origin: Point3D<T>,

    /// Z軸方向 - 法線ベクトル（STEP: axis）
    normal: Direction3D<T>,

    /// X軸方向 - 第一軸（STEP: ref_direction）
    u_axis: Direction3D<T>,

    /// Y軸方向 - 第二軸（STEP: derived, normal × u_axis）
    v_axis: Direction3D<T>,
}

impl<T: Scalar> Plane3D<T> {
    /// 原点・法線・第一軸の候補（大きさは任意）から平面を作成する
    ///
    /// 第一軸の候補から法線成分を除いて正規化する（グラム・シュミット正規直交化）。
    fn from_origin_normal_and_u_vector(
        origin: Point3D<T>,
        normal: Direction3D<T>,
        u_vector: Vector3D<T>,
    ) -> Option<Self> {
        let orthogonal_u = u_vector - normal.as_vector() * u_vector.dot(normal.as_vector());
        let u_axis = Direction3D::from_vector(orthogonal_u)?;
        // V軸 = Z軸 × U軸（右手系）
        let v_axis = Direction3D::from_vector(normal.as_vector().cross(u_axis.as_vector()))?;

        Some(Self {
            origin,
            normal,
            u_axis,
            v_axis,
        })
    }
}

impl<T: Scalar> PrimitiveMetadata for Plane3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }
}

impl<T: Scalar> TolerantEq<T> for Plane3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let dot_product = self.normal.dot(other.normal).abs();
        if (dot_product - T::ONE).abs() > tolerance {
            return false;
        }

        other.distance_to_point(self.origin) <= tolerance
    }
}

impl<T: Scalar + std::fmt::Display> std::fmt::Display for Plane3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plane3D(origin: ({}, {}, {}), normal: ({}, {}, {}), u_axis: ({}, {}, {}), v_axis: ({}, {}, {}))",
            self.origin.x(),
            self.origin.y(),
            self.origin.z(),
            self.normal.x(),
            self.normal.y(),
            self.normal.z(),
            self.u_axis.x(),
            self.u_axis.y(),
            self.u_axis.z(),
            self.v_axis.x(),
            self.v_axis.y(),
            self.v_axis.z()
        )
    }
}

impl<T: Scalar> Plane3DConstructor<T> for Plane3D<T> {
    fn from_origin_and_axes(
        origin: Point3D<T>,
        normal: Direction3D<T>,
        u_direction: Direction3D<T>,
    ) -> Option<Self> {
        Self::from_origin_normal_and_u_vector(origin, normal, u_direction.as_vector())
    }

    fn from_three_points(
        origin: Point3D<T>,
        point_u: Point3D<T>,
        point_v: Point3D<T>,
    ) -> Option<Self> {
        let u_vector = Vector3D::from_points(origin, point_u);
        let v_vector = Vector3D::from_points(origin, point_v);

        // 法線 = U × V（右手系）
        let normal = Direction3D::from_vector(u_vector.cross(v_vector))?;
        Self::from_origin_normal_and_u_vector(origin, normal, u_vector)
    }

    fn from_point_and_normal(point: Point3D<T>, normal: Direction3D<T>) -> Option<Self> {
        // 法線と平行でない座標軸を第一軸の候補にする
        let candidate_u = if normal.x().abs() < T::from_f64(0.9) {
            Vector3D::unit_x()
        } else {
            Vector3D::unit_y()
        };

        Self::from_origin_normal_and_u_vector(point, normal, candidate_u)
    }
}

impl<T: Scalar> Plane3DProperties<T> for Plane3D<T> {
    fn origin(&self) -> Point3D<T> {
        self.origin
    }

    fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    fn u_axis(&self) -> Direction3D<T> {
        self.u_axis
    }

    fn v_axis(&self) -> Direction3D<T> {
        self.v_axis
    }
}

impl<T: Scalar> Plane3DContainment<T> for Plane3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point).abs() <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Plane3DDistance<T> for Plane3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        Vector3D::from_points(self.origin, point).dot(self.normal.as_vector())
    }
}

impl<T: Scalar> Plane3DProjection<T> for Plane3D<T> {
    fn project_point(&self, point: Point3D<T>) -> Point3D<T> {
        point - self.normal.as_vector() * self.distance_to_point(point)
    }
}
