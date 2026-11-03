//! 3次元楕円体サーフェス（EllipsoidalSurface3D）のCore実装
//!
//! STEP準拠の楕円体サーフェス + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! ELLIPSOIDAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), A_RADIUS, B_RADIUS, C_RADIUS);
//! ```
//! - location: 楕円体の中心点（center）
//! - axis: Z軸方向（楕円体軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - a_radius: X軸方向の半径
//! - b_radius: Y軸方向の半径
//! - c_radius: Z軸方向の半径

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    EllipsoidalSurface3DConstructor, EllipsoidalSurface3DDistance, EllipsoidalSurface3DEvaluation,
    EllipsoidalSurface3DProperties, Scalar,
};

/// 3次元楕円体サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の楕円体サーフェス + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 楕円体の中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 楕円体軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - a_radius: X軸方向の半径
/// - b_radius: Y軸方向の半径
/// - c_radius: Z軸方向の半径
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 方位角パラメータ（経度）
/// - v ∈ [-π/2, π/2]: 仰角パラメータ（緯度）
/// - Point(u, v) = center + (a*cos(v)*cos(u)*X + b*cos(v)*sin(u)*Y + c*sin(v)*Z)
#[derive(Debug, Clone, PartialEq)]
pub struct EllipsoidalSurface3D<T: Scalar> {
    /// 楕円体の中心点（STEP: location）
    center: Point3D<T>,

    /// 楕円体の軸方向（STEP: axis）- Z軸、正規化済み
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、axis と直交
    ref_direction: Direction3D<T>,

    /// X軸方向の半径
    a_radius: T,

    /// Y軸方向の半径
    b_radius: T,

    /// Z軸方向の半径
    c_radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for EllipsoidalSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::EllipsoidalSurface
    }
}

impl<T: Scalar> EllipsoidalSurface3D<T> {
    /// Y軸方向（axis × ref_direction）
    fn y_axis(&self) -> Vector3D<T> {
        self.axis.as_vector().cross(self.ref_direction.as_vector())
    }

    /// 局所座標 (x, y, z) をワールド座標系のベクトルにする
    fn local_to_world(&self, x: T, y: T, z: T) -> Vector3D<T> {
        self.ref_direction.as_vector() * x + self.y_axis() * y + self.axis.as_vector() * z
    }

    /// 点に最も近い楕円体面上の点
    fn closest_point_on_surface(&self, point: Point3D<T>) -> Point3D<T> {
        let offset = point - self.center;
        let (x, y, z) = geo_commons::ellipsoid_closest_point(
            offset.dot(self.ref_direction.as_vector()),
            offset.dot(self.y_axis()),
            offset.dot(self.axis.as_vector()),
            self.a_radius,
            self.b_radius,
            self.c_radius,
        );
        self.center + self.local_to_world(x, y, z)
    }
}

impl<T: Scalar> EllipsoidalSurface3DConstructor<T> for EllipsoidalSurface3D<T> {
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self> {
        if a_radius <= T::ZERO || b_radius <= T::ZERO || c_radius <= T::ZERO {
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
            a_radius,
            b_radius,
            c_radius,
        })
    }

    fn new_standard(center: Point3D<T>, a_radius: T, b_radius: T, c_radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            a_radius,
            b_radius,
            c_radius,
        )
    }
}

impl<T: Scalar> EllipsoidalSurface3DProperties<T> for EllipsoidalSurface3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
    }

    fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }

    fn a_radius(&self) -> T {
        self.a_radius
    }

    fn b_radius(&self) -> T {
        self.b_radius
    }

    fn c_radius(&self) -> T {
        self.c_radius
    }
}

impl<T: Scalar> EllipsoidalSurface3DEvaluation<T> for EllipsoidalSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        self.center
            + self.local_to_world(
                self.a_radius * v.cos() * u.cos(),
                self.b_radius * v.cos() * u.sin(),
                self.c_radius * v.sin(),
            )
    }

    fn normal_at(&self, u: T, v: T) -> Direction3D<T> {
        // 陰関数 (x/a)² + (y/b)² + (z/c)² = 1 の勾配の向き。
        // cos(v) と sin(v) は同時に 0 にならないため、常に 0 でない
        Direction3D::from_vector(self.local_to_world(
            v.cos() * u.cos() / self.a_radius,
            v.cos() * u.sin() / self.b_radius,
            v.sin() / self.c_radius,
        ))
        .expect("ellipsoid gradient should be non-zero")
    }
}

impl<T: Scalar> EllipsoidalSurface3DDistance<T> for EllipsoidalSurface3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        point.distance_to(self.closest_point_on_surface(point))
    }
}

impl<T: Scalar> std::fmt::Display for EllipsoidalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EllipsoidalSurface3D {{ center: {:?}, axis: {:?}, ref_direction: {:?}, a_radius: {}, b_radius: {}, c_radius: {} }}",
            self.center,
            self.axis.as_vector(),
            self.ref_direction.as_vector(),
            self.a_radius,
            self.b_radius,
            self.c_radius
        )
    }
}
