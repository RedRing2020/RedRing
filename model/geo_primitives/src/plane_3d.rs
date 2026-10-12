//! Plane3D 実装
//!
//! 座標系付き平面実装（旧Plane3DCoordinateSystem）
//! STEP AP214準拠の完全な平面座標系を提供

mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, default_orthogonality_dot_error_tolerance, Plane3DConstructor,
    Plane3DContainment, Plane3DDistance, Plane3DProjection, Plane3DProperties, Scalar,
};

/// 3次元平面（座標系付き）
///
/// STEP AP214の AXIS2_PLACEMENT_3D + PLANE に対応
/// 完全な座標系情報（原点 + X軸 + Y軸 + Z軸）を持つ平面
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
    /// STEP AXIS2_PLACEMENT_3D 形式で平面座標系を作成
    ///
    /// # Arguments
    /// * `origin` - 平面原点
    /// * `normal` - 法線方向（Z軸）
    /// * `u_direction` - 第一軸方向（X軸）、法線と直交していなくても自動調整
    ///
    /// # Returns
    /// 新しい平面座標系、法線がゼロベクトルの場合は None
    pub fn from_origin_and_axes(
        origin: Point3D<T>,
        normal: Vector3D<T>,
        u_direction: Vector3D<T>,
    ) -> Option<Self> {
        // 法線の正規化
        let z_axis = Direction3D::from_vector(normal)?;

        // U軸の法線成分を除去して正規化（グラム・シュミット正規直交化）
        let normal_component = u_direction.dot(&z_axis.as_vector());
        let orthogonal_u = u_direction - z_axis.as_vector() * normal_component;
        let u_axis = Direction3D::from_vector(orthogonal_u)?;

        // V軸 = Z軸 × U軸（右手系）
        let v_vector = z_axis.as_vector().cross(&u_axis.as_vector());
        let v_axis = Direction3D::from_vector(v_vector)?;

        Some(Self {
            origin,
            normal: z_axis,
            u_axis,
            v_axis,
        })
    }

    /// 3点から平面座標系を作成
    ///
    /// # Arguments
    /// * `origin` - 原点
    /// * `point_u` - U軸方向の参照点
    /// * `point_v` - V軸方向の参照点
    pub fn from_three_points(
        origin: Point3D<T>,
        point_u: Point3D<T>,
        point_v: Point3D<T>,
    ) -> Option<Self> {
        let u_vector = Vector3D::new(
            point_u.x() - origin.x(),
            point_u.y() - origin.y(),
            point_u.z() - origin.z(),
        );
        let v_vector = Vector3D::new(
            point_v.x() - origin.x(),
            point_v.y() - origin.y(),
            point_v.z() - origin.z(),
        );

        // 法線 = U × V（右手系）
        let normal = u_vector.cross(&v_vector);

        Self::from_origin_and_axes(origin, normal, u_vector)
    }

    /// XY平面を作成（z = constant）
    pub fn xy_plane(z: T) -> Self {
        Self::from_origin_and_axes(
            Point3D::new(T::ZERO, T::ZERO, z),
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
        .unwrap()
    }

    /// 点と法線から平面を作成（レガシー互換性のため）
    ///
    /// # Arguments
    /// * `point` - 平面上の点（原点として使用）
    /// * `normal` - 法線ベクトル
    ///
    /// # Returns
    /// 新しい平面座標系、法線がゼロベクトルの場合は None
    pub fn from_point_and_normal(point: Point3D<T>, normal: Vector3D<T>) -> Option<Self> {
        // デフォルトのU軸候補を法線に対して垂直になるように選択
        let candidate_u = if normal.x().abs() < T::from_f64(0.9) {
            Vector3D::unit_x()
        } else {
            Vector3D::unit_y()
        };

        Self::from_origin_and_axes(point, normal, candidate_u)
    }

    /// 平面の原点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// 平面の法線方向を取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 従来の点+法線ベクトル形式で原点と法線を取得
    pub fn point(&self) -> Point3D<T> {
        self.origin
    }

    /// 点が平面上にあるかチェック
    pub fn contains_point(&self, point: Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point(point).abs();
        distance <= tolerance
    }

    /// 点から平面までの符号付き距離
    pub fn distance_to_point(&self, point: Point3D<T>) -> T {
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        relative.dot(&self.normal.as_vector())
    }

    /// 点を平面に投影
    pub fn project_point(&self, point: Point3D<T>) -> Point3D<T> {
        let distance = self.distance_to_point(point);
        let offset = self.normal.as_vector() * distance;
        Point3D::new(
            point.x() - offset.x(),
            point.y() - offset.y(),
            point.z() - offset.z(),
        )
    }

    /// 平面座標系が有効かチェック
    pub fn is_valid(&self) -> bool {
        // 各軸がDirection3Dで正規化されていることを前提
        // 直交性のチェック
        let dot_uv = self.u_axis.as_vector().dot(&self.v_axis.as_vector()).abs();
        let dot_un = self.u_axis.as_vector().dot(&self.normal.as_vector()).abs();
        let dot_vn = self.v_axis.as_vector().dot(&self.normal.as_vector()).abs();

        let tolerance = default_orthogonality_dot_error_tolerance::<T>();
        dot_uv < tolerance && dot_un < tolerance && dot_vn < tolerance
    }
}

impl<T: Scalar> Default for Plane3D<T> {
    /// デフォルトはXY平面（z = 0）
    fn default() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

impl<T: Scalar> Plane3D<T> {
    /// XY平面（z = 0）の参照
    pub fn xy() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for Plane3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::Plane
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for Plane3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let normal1 = self.normal;
        let normal2 = other.normal;

        let dot_product = normal1.dot(&normal2).abs();
        if (dot_product - T::ONE).abs() > tolerance {
            return false;
        }

        let point_on_plane1 = self.point();
        let distance = other.distance_to_point(point_on_plane1);

        distance <= tolerance
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
        origin: (T, T, T),
        normal: (T, T, T),
        u_direction: (T, T, T),
    ) -> Option<Self> {
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let u_vec = Vector3D::new(u_direction.0, u_direction.1, u_direction.2);
        Self::from_origin_and_axes(origin_point, normal_vec, u_vec)
    }

    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self> {
        let point1 = Point3D::new(p1.0, p1.1, p1.2);
        let point2 = Point3D::new(p2.0, p2.1, p2.2);
        let point3 = Point3D::new(p3.0, p3.1, p3.2);
        Self::from_three_points(point1, point2, point3)
    }

    fn from_point_and_normal(point: (T, T, T), normal: (T, T, T)) -> Option<Self> {
        let origin = Point3D::new(point.0, point.1, point.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        Self::from_point_and_normal(origin, normal_vec)
    }
}

impl<T: Scalar> Plane3DProperties<T> for Plane3D<T> {
    fn origin(&self) -> (T, T, T) {
        (self.origin.x(), self.origin.y(), self.origin.z())
    }

    fn normal(&self) -> (T, T, T) {
        (self.normal.x(), self.normal.y(), self.normal.z())
    }

    fn u_axis(&self) -> (T, T, T) {
        (self.u_axis.x(), self.u_axis.y(), self.u_axis.z())
    }

    fn v_axis(&self) -> (T, T, T) {
        (self.v_axis.x(), self.v_axis.y(), self.v_axis.z())
    }

    fn dimension(&self) -> u32 {
        2 // 2次元多様体
    }
}

impl<T: Scalar + From<f64>> Plane3DContainment<T> for Plane3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let tolerance = default_distance_tolerance::<T>();
        // distance_to_point の計算を直接展開
        let relative = Vector3D::new(
            point.0 - self.origin.x(),
            point.1 - self.origin.y(),
            point.2 - self.origin.z(),
        );
        let distance = relative.dot(&self.normal.as_vector()).abs();
        distance <= tolerance
    }
}

impl<T: Scalar + From<f64>> Plane3DDistance<T> for Plane3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let relative = Vector3D::new(
            point.0 - self.origin.x(),
            point.1 - self.origin.y(),
            point.2 - self.origin.z(),
        );
        relative.dot(&self.normal.as_vector())
    }
}

impl<T: Scalar + From<f64>> Plane3DProjection<T> for Plane3D<T> {
    fn project_point(&self, point: (T, T, T)) -> (T, T, T) {
        // distance_to_point の計算を直接展開
        let relative = Vector3D::new(
            point.0 - self.origin.x(),
            point.1 - self.origin.y(),
            point.2 - self.origin.z(),
        );
        let distance = relative.dot(&self.normal.as_vector());
        let offset = self.normal.as_vector() * distance;
        (
            point.0 - offset.x(),
            point.1 - offset.y(),
            point.2 - offset.z(),
        )
    }
}
