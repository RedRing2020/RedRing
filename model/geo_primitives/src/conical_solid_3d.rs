//! 3次元円錐ソリッド（ConicalSolid3D）のCore実装
//!
//! STEP準拠のSOLID_CONE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! SOLID_CONE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, HEIGHT);
//! ```
//! - location: 円錐の底面中心点（center）
//! - axis: Z軸方向（底面から頂点への軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 底面の半径
//! - height: 円錐の高さ（軸方向の長さ）

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D};
use geo_contracts::{
    default_distance_tolerance, ConicalSolid3DConstructor, ConicalSolid3DContainment,
    ConicalSolid3DDistance, ConicalSolid3DProperties, PointClassification, Scalar,
};

/// 3次元円錐ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の SOLID_CONE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として内部判定を持つ
///
/// ## 座標系定義（STEP準拠）
/// - center: 円錐の底面中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 底面から頂点への軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 底面の半径
/// - height: 円錐の高さ（軸方向の長さ）
#[derive(Debug, Clone, PartialEq)]
pub struct ConicalSolid3D<T: Scalar> {
    /// 円錐の底面中心点（STEP: location）
    center: Point3D<T>,

    /// 軸方向 - 底面から頂点への方向（STEP: axis）
    axis: Direction3D<T>,

    /// 参照方向 - X軸方向（STEP: ref_direction）、axis と直交
    ref_direction: Direction3D<T>,

    /// 底面の半径（正の値）
    radius: T,

    /// 円錐の高さ（正の値）
    height: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for ConicalSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::ConicalSolid
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for ConicalSolid3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let manhattan =
            |a: [T; 3], b: [T; 3]| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
        let point = |p: Point3D<T>| [p.x(), p.y(), p.z()];
        let direction = |d: Direction3D<T>| [d.x(), d.y(), d.z()];

        manhattan(point(self.center), point(other.center)) <= tolerance
            && manhattan(direction(self.axis), direction(other.axis)) <= tolerance
            && manhattan(
                direction(self.ref_direction),
                direction(other.ref_direction),
            ) <= tolerance
            && (self.radius - other.radius).abs() <= tolerance
            && (self.height - other.height).abs() <= tolerance
    }
}

impl<T: Scalar> ConicalSolid3D<T> {
    /// 点の (軸からの距離, 底面からの軸方向の距離)
    fn profile_coordinates(&self, point: Point3D<T>) -> (T, T) {
        let offset = point - self.center;
        let axial = offset.dot(self.axis.as_vector());
        let radial = (offset - self.axis.as_vector() * axial).length();
        (radial, axial)
    }

    /// 断面の上で、底面の線分と母線の線分までの距離の最小値（境界までの距離）
    fn distance_to_boundary(&self, point: Point3D<T>) -> T {
        let profile = self.profile_coordinates(point);
        let to_base = distance_to_segment_2d(profile, (T::ZERO, T::ZERO), (self.radius, T::ZERO));
        let to_slant =
            distance_to_segment_2d(profile, (self.radius, T::ZERO), (T::ZERO, self.height));
        to_base.min(to_slant)
    }
}

/// 2D の点から線分までの最短距離
fn distance_to_segment_2d<T: Scalar>(point: (T, T), start: (T, T), end: (T, T)) -> T {
    let edge = (end.0 - start.0, end.1 - start.1);
    let to_point = (point.0 - start.0, point.1 - start.1);
    let edge_length_squared = edge.0 * edge.0 + edge.1 * edge.1;
    let t = ((to_point.0 * edge.0 + to_point.1 * edge.1) / edge_length_squared)
        .max(T::ZERO)
        .min(T::ONE);
    let dx = to_point.0 - edge.0 * t;
    let dy = to_point.1 - edge.1 * t;
    (dx * dx + dy * dy).sqrt()
}

impl<T: Scalar> ConicalSolid3DConstructor<T> for ConicalSolid3D<T> {
    fn new(
        base_center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
        height: T,
    ) -> Option<Self> {
        if radius <= T::ZERO || height <= T::ZERO {
            return None;
        }

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let ref_direction = Direction3D::from_vector(
            ref_direction.as_vector() - axis.as_vector() * ref_direction.dot(axis),
        )?;

        Some(Self {
            center: base_center,
            axis,
            ref_direction,
            radius,
            height,
        })
    }

    fn new_standard(base_center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            base_center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            radius,
            height,
        )
    }
}

impl<T: Scalar> ConicalSolid3DProperties<T> for ConicalSolid3D<T> {
    fn base_center(&self) -> Point3D<T> {
        self.center
    }

    fn apex(&self) -> Point3D<T> {
        self.center + self.axis.as_vector() * self.height
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn height(&self) -> T {
        self.height
    }

    fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }
}

impl<T: Scalar> ConicalSolid3DContainment<T> for ConicalSolid3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        // 底面からの高さでの半径は、底面の半径から頂点に向かって線形に 0 になる
        let (radial, axial) = self.profile_coordinates(point);
        axial >= T::ZERO
            && axial <= self.height
            && radial <= self.radius * (T::ONE - axial / self.height)
    }

    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        if self.distance_to_boundary(point) <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> ConicalSolid3DDistance<T> for ConicalSolid3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        if self.contains_point(point) {
            T::ZERO
        } else {
            self.distance_to_boundary(point)
        }
    }
}

impl<T: Scalar> std::fmt::Display for ConicalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ConicalSolid3D(center: ({}, {}, {}), axis: ({}, {}, {}), radius: {}, height: {})",
            self.center.x(),
            self.center.y(),
            self.center.z(),
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.radius,
            self.height
        )
    }
}
