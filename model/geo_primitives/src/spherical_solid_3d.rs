//! 3次元球ソリッドのCore実装
//!
//! STEP の SOLID_SPHERE と AXIS2_PLACEMENT_3D への対応
//! 拡張機能は spherical_solid_3d_extensions.rs を参照

// use crate::{BBox3D, Direction3D, Plane3DCoordinateSystem, Point3D, Vector3D}; // 一時的にコメントアウト
use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{IntersectsRelation, Scalar};

/// STEP 準拠の 3 次元球ソリッド
///
/// 中心、参照軸、参照方向、半径を保持
/// 体積、表面積、包含判定を提供
#[derive(Debug, Clone, PartialEq)]
pub struct SphericalSolid3D<T: Scalar> {
    /// 球の中心点
    center: Point3D<T>,

    /// 参照軸方向
    axis: Direction3D<T>,

    /// 参照方向
    ref_direction: Direction3D<T>,

    /// 球の半径
    radius: T,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for SphericalSolid3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::SphericalSolid
    }
}

impl<T: Scalar> SphericalSolid3D<T> {
    /// STEP 配置情報からの球ソリッド生成
    ///
    /// 軸の正規化と参照方向の直交補正を実施
    pub fn new(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        radius: T,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        let z_axis = Direction3D::from_vector(axis)?;

        let axis_component = ref_direction.dot(&z_axis.as_vector());
        let orthogonal_ref = ref_direction - z_axis.as_vector() * axis_component;

        let x_axis = Direction3D::from_vector(orthogonal_ref)?;

        Some(Self {
            center,
            axis: z_axis,
            ref_direction: x_axis,
            radius,
        })
    }

    /// 標準軸での球ソリッド生成
    pub fn new_standard(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
        )
    }

    /// 原点中心の球ソリッド生成
    pub fn new_at_origin(radius: T) -> Option<Self> {
        Self::new_standard(Point3D::new(T::ZERO, T::ZERO, T::ZERO), radius)
    }

    /// 直径からの球ソリッド生成
    pub fn from_diameter(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        diameter: T,
    ) -> Option<Self> {
        if diameter <= T::ZERO {
            return None;
        }
        Self::new(center, axis, ref_direction, diameter / T::from_f64(2.0))
    }

    /// 球の中心点の取得
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 参照軸方向の取得
    pub(crate) fn axis_internal(&self) -> Direction3D<T> {
        self.axis
    }

    /// 参照方向の取得
    pub(crate) fn ref_direction_internal(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// Y 軸相当方向の計算
    #[allow(dead_code)]
    pub(crate) fn y_axis_internal(&self) -> Direction3D<T> {
        let y_vector = self.axis.as_vector().cross(&self.ref_direction.as_vector());
        Direction3D::from_vector(y_vector)
            .expect("Y-axis calculation should always succeed with orthogonal axes")
    }

    /// 半径の取得
    pub(crate) fn radius_internal(&self) -> T {
        self.radius
    }

    // 一時的にコメントアウト: Plane3DCoordinateSystemが未定義
    // pub fn position(&self) -> Plane3DCoordinateSystem<T> {
    //     // Plane3DCoordinateSystemを原点と軸から構築
    //     Plane3DCoordinateSystem::from_origin_and_axes(
    //         self.center,
    //         self.axis.as_vector(),          // 法線方向
    //         self.ref_direction.as_vector(), // U軸方向
    //     )
    //     .expect("Coordinate system creation should succeed with valid sphere parameters")
    // }
    /// 直径の取得
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    /// 体積の計算
    pub fn volume(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius * self.radius / T::from_f64(3.0)
    }

    /// 表面積の計算
    pub fn surface_area(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius
    }

    /// 境界ボックスの計算
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        let min_point = geo_core::Point3D::new(
            self.center.x() - self.radius,
            self.center.y() - self.radius,
            self.center.z() - self.radius,
        );
        let max_point = geo_core::Point3D::new(
            self.center.x() + self.radius,
            self.center.y() + self.radius,
            self.center.z() + self.radius,
        );
        geo_core::Aabb3D::new(min_point, max_point)
    }

    /// 点の内部包含判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        let distance_squared = self.center.distance_squared_to(&point);
        distance_squared <= self.radius * self.radius
    }

    /// 点から表面までの符号付き距離の返却
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let distance_to_center = self.center.distance_to(&point);
        distance_to_center - self.radius
    }

    /// 退化判定
    pub fn is_degenerate(&self) -> bool {
        self.radius <= T::EPSILON
    }

    /// 無限直線までの最短距離の返却
    pub fn distance_to_infinite_line(
        &self,
        line_point: &Point3D<T>,
        line_direction: &Vector3D<T>,
    ) -> T {
        let to_center = self.center - *line_point;
        let dir_dot = line_direction.dot(line_direction);
        let t = to_center.dot(line_direction) / dir_dot;
        let closest = *line_point + *line_direction * t;
        let distance_from_center = (self.center - closest).length();

        if distance_from_center <= self.radius {
            T::ZERO
        } else {
            distance_from_center - self.radius
        }
    }

    /// 光線までの最短距離の返却
    pub fn distance_to_ray(&self, ray_origin: &Point3D<T>, ray_direction: &Vector3D<T>) -> T {
        let to_center = self.center - *ray_origin;
        let dir_dot = ray_direction.dot(ray_direction);
        let t = to_center.dot(ray_direction) / dir_dot;

        if t < T::ZERO {
            let dist_to_origin = to_center.length();
            if dist_to_origin <= self.radius {
                T::ZERO
            } else {
                dist_to_origin - self.radius
            }
        } else {
            let closest = *ray_origin + *ray_direction * t;
            let distance_from_center = (self.center - closest).length();
            if distance_from_center <= self.radius {
                T::ZERO
            } else {
                distance_from_center - self.radius
            }
        }
    }

    /// 線分までの最短距離の返却
    pub fn distance_to_line_segment(
        &self,
        segment_start: &Point3D<T>,
        segment_end: &Point3D<T>,
    ) -> T {
        let direction = *segment_end - *segment_start;
        let to_center = self.center - *segment_start;
        let dir_dot = direction.dot(&direction);
        let t = to_center.dot(&direction) / dir_dot;

        let nearest = if t < T::ZERO {
            *segment_start
        } else if t > T::ONE {
            *segment_end
        } else {
            *segment_start + direction * t
        };

        let distance_from_center = (self.center - nearest).length();
        if distance_from_center <= self.radius {
            T::ZERO
        } else {
            distance_from_center - self.radius
        }
    }

    /// 指定方向の表面点の返却
    pub fn point_on_surface_in_direction(&self, direction: Vector3D<T>) -> Option<Point3D<T>> {
        let normalized = direction.normalize();
        if normalized == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        Some(Point3D::new(
            self.center.x() + normalized.x() * self.radius,
            self.center.y() + normalized.y() * self.radius,
            self.center.z() + normalized.z() * self.radius,
        ))
    }

    /// 指定点方向の表面点の返却
    pub fn point_on_surface_towards(&self, target: Point3D<T>) -> Option<Point3D<T>> {
        let direction = Vector3D::new(
            target.x() - self.center.x(),
            target.y() - self.center.y(),
            target.z() - self.center.z(),
        );
        self.point_on_surface_in_direction(direction)
    }
}

use geo_contracts::{
    SphericalSolid3DConstructor, SphericalSolid3DContainment, SphericalSolid3DDerived,
    SphericalSolid3DDistance, SphericalSolid3DEvaluation, SphericalSolid3DProjection,
    SphericalSolid3DProperties as ContractsSphericalSolid3DProperties,
};

impl<T: Scalar> SphericalSolid3DConstructor<T> for SphericalSolid3D<T> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_vector = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_vector = Vector3D::new(ref_direction.0, ref_direction.1, ref_direction.2);
        Self::new(center_point, axis_vector, ref_vector, radius)
    }

    fn new_standard(center: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_standard(center_point, radius)
    }

    fn unit_sphere() -> Self {
        Self::new_at_origin(T::ONE).expect("Unit sphere creation should always succeed")
    }

    fn from_diameter(center: (T, T, T), diameter: T) -> Option<Self> {
        let radius = diameter / (T::ONE + T::ONE);
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_standard(center_point, radius)
    }

    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self> {
        let center_x = (min.0 + max.0) / (T::ONE + T::ONE);
        let center_y = (min.1 + max.1) / (T::ONE + T::ONE);
        let center_z = (min.2 + max.2) / (T::ONE + T::ONE);

        let dx = (max.0 - min.0) / (T::ONE + T::ONE);
        let dy = (max.1 - min.1) / (T::ONE + T::ONE);
        let dz = (max.2 - min.2) / (T::ONE + T::ONE);

        let radius = dx.min(dy).min(dz);
        let center_point = Point3D::new(center_x, center_y, center_z);
        Self::new_standard(center_point, radius)
    }

    fn from_four_points(
        p1: (T, T, T),
        p2: (T, T, T),
        p3: (T, T, T),
        p4: (T, T, T),
    ) -> Option<Self> {
        // 厳密外接球ではなく重心ベースの近似生成
        let center_x = (p1.0 + p2.0 + p3.0 + p4.0) / T::from_f64(4.0);
        let center_y = (p1.1 + p2.1 + p3.1 + p4.1) / T::from_f64(4.0);
        let center_z = (p1.2 + p2.2 + p3.2 + p4.2) / T::from_f64(4.0);

        let dist1 =
            ((p1.0 - center_x).powi(2) + (p1.1 - center_y).powi(2) + (p1.2 - center_z).powi(2))
                .sqrt();
        let dist2 =
            ((p2.0 - center_x).powi(2) + (p2.1 - center_y).powi(2) + (p2.2 - center_z).powi(2))
                .sqrt();
        let dist3 =
            ((p3.0 - center_x).powi(2) + (p3.1 - center_y).powi(2) + (p3.2 - center_z).powi(2))
                .sqrt();
        let dist4 =
            ((p4.0 - center_x).powi(2) + (p4.1 - center_y).powi(2) + (p4.2 - center_z).powi(2))
                .sqrt();

        let radius = dist1.max(dist2).max(dist3).max(dist4);
        let center_point = Point3D::new(center_x, center_y, center_z);
        Self::new_standard(center_point, radius)
    }
}

impl<T: Scalar> ContractsSphericalSolid3DProperties<T> for SphericalSolid3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.center_internal();
        (c.x(), c.y(), c.z())
    }

    fn radius(&self) -> T {
        self.radius_internal()
    }

    fn axis(&self) -> (T, T, T) {
        let a = self.axis_internal();
        (a.x(), a.y(), a.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        let r = self.ref_direction_internal();
        (r.x(), r.y(), r.z())
    }

    fn diameter(&self) -> T {
        self.radius_internal() * T::from_f64(2.0)
    }

    fn is_unit_sphere(&self) -> bool {
        (self.radius_internal() - T::ONE).abs() <= T::EPSILON
    }

    fn circumference(&self) -> T {
        T::TAU * self.radius_internal()
    }

    fn is_centered_at_origin(&self) -> bool {
        let c = self.center_internal();
        c.x().abs() <= T::EPSILON && c.y().abs() <= T::EPSILON && c.z().abs() <= T::EPSILON
    }
}

impl<T: Scalar> SphericalSolid3DDerived<T> for SphericalSolid3D<T> {
    fn volume(&self) -> T {
        self.volume()
    }

    fn surface_area(&self) -> T {
        self.surface_area()
    }

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let c = self.center_internal();
        let r = self.radius_internal();

        let min = (c.x() - r, c.y() - r, c.z() - r);
        let max = (c.x() + r, c.y() + r, c.z() + r);

        (min, max)
    }
}

impl<T: Scalar> SphericalSolid3DContainment<T> for SphericalSolid3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let point_3d = Point3D::new(point.0, point.1, point.2);
        self.contains_point(point_3d)
    }
}

impl<T: Scalar> SphericalSolid3DDistance<T> for SphericalSolid3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let point_3d = Point3D::new(point.0, point.1, point.2);
        self.distance_to_surface(point_3d).abs()
    }
}

impl<T: Scalar> SphericalSolid3DEvaluation<T> for SphericalSolid3D<T> {
    fn point_at_latlong(&self, latitude: T, longitude: T) -> (T, T, T) {
        let c = self.center_internal();
        let r = self.radius_internal();

        let cos_lat = latitude.cos();
        let sin_lat = latitude.sin();
        let cos_lon = longitude.cos();
        let sin_lon = longitude.sin();

        let x = c.x() + r * cos_lat * cos_lon;
        let y = c.y() + r * cos_lat * sin_lon;
        let z = c.z() + r * sin_lat;

        (x, y, z)
    }
}

impl<T: Scalar> SphericalSolid3DProjection<T> for SphericalSolid3D<T> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T) {
        let c = self.center_internal();
        let r = self.radius_internal();
        let p = Point3D::new(point.0, point.1, point.2);

        let dir = Vector3D::from_points(&c, &p);
        let len = dir.length();

        if len < T::EPSILON {
            return (c.x() + r, c.y(), c.z());
        }

        let normalized = dir / len;
        let surface_point = Point3D::new(
            c.x() + normalized.x() * r,
            c.y() + normalized.y() * r,
            c.z() + normalized.z() * r,
        );

        (surface_point.x(), surface_point.y(), surface_point.z())
    }
}

impl<T: Scalar> IntersectsRelation<Self> for SphericalSolid3D<T> {
    fn intersects(&self, other: &Self) -> bool {
        let center = other.center_internal();
        let radius = other.radius_internal();
        self.center_internal().distance_to(&center) <= (self.radius_internal() + radius)
    }
}

impl<T: Scalar> std::fmt::Display for SphericalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SphericalSolid3D(center: ({}, {}, {}), axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {})",
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

/// 旧名前との互換性のためのエイリアス
/// 将来のバージョンで削除予定
#[deprecated(since = "0.1.0", note = "Use SphericalSolid3D instead")]
pub type SolidSphere3D<T> = SphericalSolid3D<T>;
