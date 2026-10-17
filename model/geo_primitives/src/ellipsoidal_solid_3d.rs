//! 3次元楕円体ソリッド（EllipsoidalSolid3D）のCore実装
//!
//! STEP準拠の ELLIPSOIDAL_SOLID + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//!
//! ## STEP標準対応
//! ```step
//! ELLIPSOIDAL_SOLID('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), A_RADIUS, B_RADIUS, C_RADIUS);
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
    default_distance_tolerance, EllipsoidalSolid3DConstructor, EllipsoidalSolid3DContainment,
    EllipsoidalSolid3DDistance, EllipsoidalSolid3DProjection, EllipsoidalSolid3DProperties,
    PointClassification, Scalar,
};

/// 3次元楕円体ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の ELLIPSOIDAL_SOLID + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として体積・内部判定を持つ
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
/// ## ソリッド特性
/// - 体積計算：V = (4/3)π × a × b × c
/// - 内部判定：点の包含テスト (x/a)² + (y/b)² + (z/c)² ≤ 1
/// - 表面積計算：Knudの近似式を使用
#[derive(Debug, Clone, PartialEq)]
pub struct EllipsoidalSolid3D<T: Scalar> {
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

impl<T: Scalar> geo_contracts::PrimitiveMetadata for EllipsoidalSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::EllipsoidalSolid
    }
}

impl<T: Scalar> EllipsoidalSolid3D<T> {
    /// Y軸方向（axis × ref_direction）
    fn y_axis(&self) -> Vector3D<T> {
        self.axis.as_vector().cross(self.ref_direction.as_vector())
    }

    /// 点の局所座標 (x, y, z)
    fn local_coordinates(&self, point: Point3D<T>) -> (T, T, T) {
        let offset = point - self.center;
        (
            offset.dot(self.ref_direction.as_vector()),
            offset.dot(self.y_axis()),
            offset.dot(self.axis.as_vector()),
        )
    }

    /// 点から表面までの距離（内部・外部とも正）
    fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let (x, y, z) = self.local_coordinates(point);
        let (surface_x, surface_y, surface_z) = geo_commons::ellipsoid_closest_point(
            x,
            y,
            z,
            self.a_radius,
            self.b_radius,
            self.c_radius,
        );
        Vector3D::new(x - surface_x, y - surface_y, z - surface_z).length()
    }
}

impl<T: Scalar> EllipsoidalSolid3DConstructor<T> for EllipsoidalSolid3D<T> {
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

impl<T: Scalar> EllipsoidalSolid3DProperties<T> for EllipsoidalSolid3D<T> {
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

impl<T: Scalar> EllipsoidalSolid3DContainment<T> for EllipsoidalSolid3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        // (x/a)² + (y/b)² + (z/c)² ≤ 1
        let (x, y, z) = self.local_coordinates(point);
        let x = x / self.a_radius;
        let y = y / self.b_radius;
        let z = z / self.c_radius;
        x * x + y * y + z * z <= T::ONE
    }

    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        if self.distance_to_surface(point) <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> EllipsoidalSolid3DProjection<T> for EllipsoidalSolid3D<T> {
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        if self.contains_point(point) {
            return point;
        }
        let (x, y, z) = self.local_coordinates(point);
        let (surface_x, surface_y, surface_z) = geo_commons::ellipsoid_closest_point(
            x,
            y,
            z,
            self.a_radius,
            self.b_radius,
            self.c_radius,
        );
        self.center
            + self.ref_direction.as_vector() * surface_x
            + self.y_axis() * surface_y
            + self.axis.as_vector() * surface_z
    }
}

impl<T: Scalar> EllipsoidalSolid3DDistance<T> for EllipsoidalSolid3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        if self.contains_point(point) {
            T::ZERO
        } else {
            self.distance_to_surface(point)
        }
    }
}
