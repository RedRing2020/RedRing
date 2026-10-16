//! 3次元円柱サーフェス（CylindricalSurface3D）のCore実装
//!
//! STEP準拠のCYLINDRICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! CYLINDRICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS);
//! ```
//! - location: 円柱軸上の基準点（center）
//! - axis: Z軸方向（円柱軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 円柱半径
//! - 高さは境界によって定義（無限円柱面）

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    CylindricalSurface3DConstructor, CylindricalSurface3DDistance, CylindricalSurface3DEvaluation,
    CylindricalSurface3DProperties, Scalar,
};

/// 3次元円柱サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の CYLINDRICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 円柱軸上の基準点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 円柱軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 円柱半径
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 円周方向パラメータ（角度）
/// - v ∈ ℝ: 軸方向パラメータ（無限範囲、境界で制限）
/// - Point(u, v) = center + radius * (cos(u) * X + sin(u) * Y) + v * Z
#[derive(Debug, Clone, PartialEq)]
pub struct CylindricalSurface3D<T: Scalar> {
    /// 円柱軸上の基準点（STEP: location）
    center: Point3D<T>,

    /// 円柱の軸方向（STEP: axis）- Z軸、正規化済み
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、axis と直交
    ref_direction: Direction3D<T>,

    /// 円柱の半径
    radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for CylindricalSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::CylindricalSurface
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for CylindricalSurface3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 軸は向きが逆でも同じ円柱面
        let tolerance_squared = tolerance * tolerance;
        let axis = self.axis.as_vector();
        let other_axis = other.axis.as_vector();
        self.center.distance_squared_to(other.center) <= tolerance_squared
            && ((axis - other_axis).length_squared() <= tolerance_squared
                || (axis + other_axis).length_squared() <= tolerance_squared)
            && (self.ref_direction.as_vector() - other.ref_direction.as_vector()).length_squared()
                <= tolerance_squared
            && (self.radius - other.radius).abs() <= tolerance
    }
}

impl<T: Scalar> CylindricalSurface3D<T> {
    /// Y軸方向（axis × ref_direction）
    fn y_axis(&self) -> Vector3D<T> {
        self.axis.as_vector().cross(self.ref_direction.as_vector())
    }

    /// 円周方向の角度 `u` の、軸から離れる向きの単位ベクトル
    fn radial_direction_at(&self, u: T) -> Vector3D<T> {
        self.ref_direction.as_vector() * u.cos() + self.y_axis() * u.sin()
    }
}

impl<T: Scalar> CylindricalSurface3DConstructor<T> for CylindricalSurface3D<T> {
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

impl<T: Scalar> CylindricalSurface3DProperties<T> for CylindricalSurface3D<T> {
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

impl<T: Scalar> CylindricalSurface3DEvaluation<T> for CylindricalSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        self.center + self.radial_direction_at(u) * self.radius + self.axis.as_vector() * v
    }

    fn normal_at(&self, u: T, _v: T) -> Direction3D<T> {
        Direction3D::from_vector(self.radial_direction_at(u))
            .expect("radial direction should be a unit vector")
    }
}

impl<T: Scalar> CylindricalSurface3DDistance<T> for CylindricalSurface3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        let offset = point - self.center;
        let axial = offset.dot(self.axis.as_vector());
        let radial = (offset - self.axis.as_vector() * axial).length();
        (radial - self.radius).abs()
    }
}

impl<T: Scalar> std::fmt::Display for CylindricalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CylindricalSurface3D(center: {:?}, axis: {:?}, ref_direction: {:?}, radius: {:?})",
            self.center,
            self.axis.as_vector(),
            self.ref_direction.as_vector(),
            self.radius
        )
    }
}
