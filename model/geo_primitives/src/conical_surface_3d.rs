//! 3次元円錐サーフェス（ConicalSurface3D）のCore実装
//!
//! STEP準拠のCONICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! CONICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, SEMI_ANGLE);
//! ```
//! - location: 円錐軸上の基準点（center）
//! - axis: Z軸方向（円錐軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 基準点での半径
//! - semi_angle: 半頂角

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, Angle, ConicalSurface3DConstructor, ConicalSurface3DDistance,
    ConicalSurface3DEvaluation, ConicalSurface3DProperties, Scalar,
};

/// 3次元円錐サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の CONICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 円錐軸上の基準点（STEP: location）- 基準半径位置
/// - axis: Z軸方向（STEP: axis）- 円錐軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 基準点での半径
/// - semi_angle: 半頂角 - 円錐の開き角度
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 円周方向パラメータ（角度）
/// - v ∈ ℝ: 軸方向パラメータ（無限範囲、境界で制限）
/// - Point(u, v) = center + (radius + v * tan(semi_angle)) * (cos(u) * X + sin(u) * Y) + v * Z
/// - 頂点（v = -radius / tan(semi_angle)）の先では半径が負になり、頂点の反対側の円錐を表す
#[derive(Debug, Clone, PartialEq)]
pub struct ConicalSurface3D<T: Scalar> {
    /// 円錐軸上の基準点（STEP: location）
    center: Point3D<T>,

    /// 円錐の軸方向（STEP: axis）- Z軸、正規化済み
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、axis と直交
    ref_direction: Direction3D<T>,

    /// 基準点での半径
    radius: T,

    /// 半頂角
    semi_angle: Angle<T>,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for ConicalSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::ConicalSurface
    }
}

impl<T: Scalar> ConicalSurface3D<T> {
    /// Y軸方向（axis × ref_direction）
    fn y_axis(&self) -> Vector3D<T> {
        self.axis.as_vector().cross(self.ref_direction.as_vector())
    }

    /// 円周方向の角度 `u` の、軸から離れる向きの単位ベクトル
    fn radial_direction_at(&self, u: T) -> Vector3D<T> {
        self.ref_direction.as_vector() * u.cos() + self.y_axis() * u.sin()
    }

    /// 軸方向の距離 `v` での半径（頂点の先では負）
    fn radius_at(&self, v: T) -> T {
        self.radius + v * self.semi_angle.tan()
    }

    /// 基準点から頂点までの軸方向の距離（負の値）
    fn apex_parameter(&self) -> T {
        -self.radius / self.semi_angle.tan()
    }
}

/// 2D の点から、始点と単位方向で表す半直線までの距離
fn distance_to_ray_2d<T: Scalar>(point: (T, T), origin: (T, T), direction: (T, T)) -> T {
    let to_point = (point.0 - origin.0, point.1 - origin.1);
    let t = (to_point.0 * direction.0 + to_point.1 * direction.1).max(T::ZERO);
    let dx = to_point.0 - direction.0 * t;
    let dy = to_point.1 - direction.1 * t;
    (dx * dx + dy * dy).sqrt()
}

impl<T: Scalar> ConicalSurface3DConstructor<T> for ConicalSurface3D<T> {
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
        semi_angle: Angle<T>,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // 半頂角は 0 < semi_angle < π/2
        let half_pi = T::PI / (T::ONE + T::ONE);
        if semi_angle.to_radians() <= T::ZERO || semi_angle.to_radians() >= half_pi {
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
            semi_angle,
        })
    }

    fn new_standard(center: Point3D<T>, radius: T, semi_angle: Angle<T>) -> Option<Self> {
        Self::new(
            center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            radius,
            semi_angle,
        )
    }
}

impl<T: Scalar> ConicalSurface3DProperties<T> for ConicalSurface3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn semi_angle(&self) -> Angle<T> {
        self.semi_angle
    }

    fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }

    fn apex(&self) -> Point3D<T> {
        self.center + self.axis.as_vector() * self.apex_parameter()
    }
}

impl<T: Scalar> ConicalSurface3DEvaluation<T> for ConicalSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        self.center + self.radial_direction_at(u) * self.radius_at(v) + self.axis.as_vector() * v
    }

    fn normal_at(&self, u: T, v: T) -> Option<Direction3D<T>> {
        // ∂u × ∂v = radius(v) * (径方向 - tan(semi_angle) * 軸方向)。頂点（半径 0）では定まらない
        let radius = self.radius_at(v);
        if radius.abs() <= default_distance_tolerance::<T>() {
            return None;
        }
        let generator_normal =
            self.radial_direction_at(u) - self.axis.as_vector() * self.semi_angle.tan();
        Direction3D::from_vector(generator_normal * radius)
    }
}

impl<T: Scalar> ConicalSurface3DDistance<T> for ConicalSurface3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        // 点と軸を含む断面の上で、頂点から両側に延びる母線（半直線）までの距離の最小値
        let offset = point - self.center;
        let axial = offset.dot(self.axis.as_vector());
        let radial = (offset - self.axis.as_vector() * axial).length();
        let apex = (T::ZERO, self.apex_parameter());
        let (sin, cos) = (self.semi_angle.sin(), self.semi_angle.cos());
        distance_to_ray_2d((radial, axial), apex, (sin, cos)).min(distance_to_ray_2d(
            (radial, axial),
            apex,
            (sin, -cos),
        ))
    }
}

impl<T: Scalar> std::fmt::Display for ConicalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ConicalSurface3D {{ center: {:?}, axis: {:?}, ref_direction: {:?}, radius: {}, semi_angle: {} }}",
            self.center,
            self.axis.as_vector(),
            self.ref_direction.as_vector(),
            self.radius,
            self.semi_angle.to_radians()
        )
    }
}
