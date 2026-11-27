mod bounds;
#[cfg(test)]
mod tests;
mod transform;

// torus_surface_3d.rs
// STEP AP214 準拠のトーラス面実装
//
// トーラス面は大半径（major_radius）と小半径（minor_radius）を持つ回転面です。
//
// STEP エンティティ: TOROIDAL_SURFACE + AXIS2_PLACEMENT_3D
// パラメータ範囲: u ∈ [0, 2π], v ∈ [0, 2π]（管が軸と交わるトーラスでは、v は R + r cos v ≥ 0 の範囲）

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    Scalar, TorusSurface3DConstructor, TorusSurface3DDistance, TorusSurface3DEvaluation,
    TorusSurface3DProperties,
};

/// STEP AP214 準拠のトーラス面
///
/// 大半径（中心軸から断面の円の中心までの距離）と
/// 小半径（断面の円の半径）により定義されます。
///
/// 面は、点 C + (R + r cos v)(cos u X + sin u Y) + r sin v Z のうち、R + r cos v ≥ 0 の部分
/// （断面の円の、軸から見て角度 u の側にある部分）とする。大半径が小半径より大きければトーラスの全体、
/// 大半径が正で小半径以下なら外側のりんご形、大半径が負なら内側のレモン形（管の中心が軸の向こう側にある）。
/// 管が軸と交わるトーラスでは、R + r cos v = 0 の 2 つの v で、面が軸の上の 1 点になる。
#[derive(Debug, Clone, PartialEq)]
pub struct TorusSurface3D<T: Scalar> {
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

impl<T: Scalar> geo_contracts::PrimitiveMetadata for TorusSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::TorusSurface
    }
}

impl<T: Scalar> TorusSurface3D<T> {
    /// Y軸方向（z_axis × x_axis）
    fn y_axis(&self) -> Vector3D<T> {
        self.z_axis.as_vector().cross(self.x_axis.as_vector())
    }

    /// 軸回りの角度 `u` の、軸から離れる向きの単位ベクトル
    fn radial_direction_at(&self, u: T) -> Vector3D<T> {
        self.x_axis.as_vector() * u.cos() + self.y_axis() * u.sin()
    }
}

impl<T: Scalar> TorusSurface3DConstructor<T> for TorusSurface3D<T> {
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        // 大半径が 0 なら球、−小半径以下なら面がない（または 1 点）
        if minor_radius <= T::ZERO || major_radius == T::ZERO || major_radius <= -minor_radius {
            return None;
        }

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let x_axis = ref_direction.orthogonal_to(axis)?;

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

impl<T: Scalar> TorusSurface3DProperties<T> for TorusSurface3D<T> {
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

impl<T: Scalar> TorusSurface3DEvaluation<T> for TorusSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        let radial = self.radial_direction_at(u);
        self.origin
            + radial * (self.major_radius + self.minor_radius * v.cos())
            + self.z_axis.as_vector() * (self.minor_radius * v.sin())
    }

    fn normal_at(&self, u: T, v: T) -> Direction3D<T> {
        // 断面の円の中心から面上の点へ向かう単位ベクトル
        Direction3D::from_vector(
            self.radial_direction_at(u) * v.cos() + self.z_axis.as_vector() * v.sin(),
        )
        .expect("torus normal should be a unit vector")
    }
}

impl<T: Scalar> TorusSurface3DDistance<T> for TorusSurface3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        // 点と軸を含む断面の、点の側の半平面の上で、断面の円の軸の側（軸からの距離が 0 以上）の弧までの距離
        let local = point - self.origin;
        let axial = local.dot(self.z_axis.as_vector());
        let radial = (local - self.z_axis.as_vector() * axial).length();
        let tube_center_distance =
            Vector3D::new(radial - self.major_radius, axial, T::ZERO).length();
        if tube_center_distance == T::ZERO {
            return self.minor_radius;
        }
        // 断面の円の上の最も近い点が、軸の向こう側に出るなら、弧の端（軸の上の点）が最も近い
        let nearest_radial = self.major_radius
            + self.minor_radius * (radial - self.major_radius) / tube_center_distance;
        if self.major_radius >= self.minor_radius || nearest_radial >= T::ZERO {
            return (tube_center_distance - self.minor_radius).abs();
        }
        let pole_height =
            (self.minor_radius * self.minor_radius - self.major_radius * self.major_radius).sqrt();
        Vector3D::new(radial, axial.abs() - pole_height, T::ZERO).length()
    }
}
