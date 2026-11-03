//! 3次元球サーフェス（SphericalSurface3D）のCore実装
//!
//! STEP準拠のSPHERICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! SPHERICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS);
//! ```
//! - location: 球の中心点（center）
//! - axis: Z軸方向（参照軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 球の半径

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    Scalar, SphericalSurface3DConstructor, SphericalSurface3DDistance,
    SphericalSurface3DEvaluation, SphericalSurface3DProperties,
};

/// 3次元球サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の SPHERICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 球の中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 参照軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 球の半径
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 方位角パラメータ（経度）
/// - v ∈ [-π/2, π/2]: 仰角パラメータ（緯度）
/// - Point(u, v) = center + radius * (cos(v)*cos(u)*X + cos(v)*sin(u)*Y + sin(v)*Z)
#[derive(Debug, Clone, PartialEq)]
pub struct SphericalSurface3D<T: Scalar> {
    /// 球の中心点（STEP: location）
    center: Point3D<T>,

    /// 参照軸方向（STEP: axis）- Z軸、正規化済み
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、axis と直交
    ref_direction: Direction3D<T>,

    /// 球の半径
    radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for SphericalSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::SphericalSurface
    }
}

impl<T: Scalar> SphericalSurface3D<T> {
    /// Y軸方向（axis × ref_direction）
    fn y_axis(&self) -> Vector3D<T> {
        self.axis.as_vector().cross(self.ref_direction.as_vector())
    }

    /// 経度 `u`・緯度 `v` の方向の単位ベクトル
    fn unit_direction_at(&self, u: T, v: T) -> Vector3D<T> {
        self.ref_direction.as_vector() * (v.cos() * u.cos())
            + self.y_axis() * (v.cos() * u.sin())
            + self.axis.as_vector() * v.sin()
    }
}

impl<T: Scalar> SphericalSurface3DConstructor<T> for SphericalSurface3D<T> {
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

impl<T: Scalar> SphericalSurface3DProperties<T> for SphericalSurface3D<T> {
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

impl<T: Scalar> SphericalSurface3DEvaluation<T> for SphericalSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        self.center + self.unit_direction_at(u, v) * self.radius
    }

    fn normal_at(&self, u: T, v: T) -> Direction3D<T> {
        Direction3D::from_vector(self.unit_direction_at(u, v))
            .expect("unit direction on sphere should be normalizable")
    }
}

impl<T: Scalar> SphericalSurface3DDistance<T> for SphericalSurface3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        (self.center.distance_to(point) - self.radius).abs()
    }
}

impl<T: Scalar> std::fmt::Display for SphericalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SphericalSurface3D(center: ({}, {}, {}), axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {})",
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
