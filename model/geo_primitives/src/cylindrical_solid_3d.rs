//! 3次元円柱ソリッド（CylindricalSolid3D）のCore実装
//!
//! STEP準拠のSOLID_CYLINDER + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! SOLID_CYLINDER('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, HEIGHT);
//! ```
//! - location: 円柱底面中心（center）
//! - axis: Z軸方向（円柱軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 円柱半径
//! - height: 円柱高さ

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D};
use geo_contracts::{
    default_distance_tolerance, CylindricalSolid3DConstructor, CylindricalSolid3DContainment,
    CylindricalSolid3DDistance, CylindricalSolid3DProperties, PointClassification, Scalar,
};

/// 3次元円柱ソリッド（STEP準拠のCore実装）
///
/// 底面の中心から軸方向に高さだけ延びる円柱
#[derive(Debug, Clone, PartialEq)]
pub struct CylindricalSolid3D<T: Scalar> {
    /// 底面の中心（STEP: location）
    center: Point3D<T>,

    /// 底面から上面へ向かう軸方向（STEP: axis）
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- axis と直交
    ref_direction: Direction3D<T>,

    /// 円柱の半径
    radius: T,

    /// 円柱の高さ
    height: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for CylindricalSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::CylindricalSolid
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for CylindricalSolid3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let axis_similar = self.axis.dot(other.axis).abs() >= T::from_f64(0.999);
        let ref_similar = self.ref_direction.dot(other.ref_direction).abs() >= T::from_f64(0.999);

        self.center.distance_to(other.center) <= tolerance
            && (self.radius - other.radius).abs() <= tolerance
            && (self.height - other.height).abs() <= tolerance
            && axis_similar
            && ref_similar
    }
}

impl<T: Scalar> CylindricalSolid3D<T> {
    /// 点の (軸からの距離, 底面からの軸方向の距離)
    fn profile_coordinates(&self, point: Point3D<T>) -> (T, T) {
        let offset = point - self.center;
        let axial = offset.dot(self.axis.as_vector());
        let radial = (offset - self.axis.as_vector() * axial).length();
        (radial, axial)
    }
}

impl<T: Scalar> CylindricalSolid3DConstructor<T> for CylindricalSolid3D<T> {
    fn new(
        center: Point3D<T>,
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
            center,
            axis,
            ref_direction,
            radius,
            height,
        })
    }

    fn new_standard(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            radius,
            height,
        )
    }
}

impl<T: Scalar> CylindricalSolid3DProperties<T> for CylindricalSolid3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
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

impl<T: Scalar> CylindricalSolid3DContainment<T> for CylindricalSolid3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        let (radial, axial) = self.profile_coordinates(point);
        axial >= T::ZERO && axial <= self.height && radial <= self.radius
    }

    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        let inside = self.contains_point(point);
        let boundary_distance = if inside {
            // 内部の点は、側面・底面・上面までの距離の最小値
            let (radial, axial) = self.profile_coordinates(point);
            (self.radius - radial).min(axial).min(self.height - axial)
        } else {
            self.distance_to_point(point)
        };

        if boundary_distance <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if inside {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> CylindricalSolid3DDistance<T> for CylindricalSolid3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        // 軸方向・径方向それぞれの、立体の範囲からのはみ出しを合成する
        let (radial, axial) = self.profile_coordinates(point);
        let axial_excess = if axial < T::ZERO {
            -axial
        } else if axial > self.height {
            axial - self.height
        } else {
            T::ZERO
        };
        let radial_excess = (radial - self.radius).max(T::ZERO);
        (axial_excess * axial_excess + radial_excess * radial_excess).sqrt()
    }
}

impl<T: Scalar> std::fmt::Display for CylindricalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CylindricalSolid3D(center: {:?}, axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {}, height: {})",
            self.center,
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
            self.radius,
            self.height
        )
    }
}
