//! 3次元円（Circle3D）の実装
//!
//! STEP (ISO 10303) 準拠の axis2_placement_3d スタイルで実装

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, default_kernel_numerical_zero_tolerance,
    Circle3DConstructor, Circle3DContainment, Circle3DDistance, Circle3DEvaluation,
    Circle3DProjection, Circle3DProperties, CrossDistance, PointClassification, PrimitiveKind,
    PrimitiveMetadata, Scalar, TolerantEq,
};

/// 3次元空間の円
///
/// STEP準拠でaxis（法線）とref_direction（参照方向）を持つ
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle3D<T: Scalar> {
    center: Point3D<T>,
    /// Z軸方向（円が存在する平面の法線ベクトル）
    axis: Direction3D<T>,
    /// X軸方向（参照方向、角度0度の方向）
    ref_direction: Direction3D<T>,
    radius: T,
}

impl<T: Scalar> PrimitiveMetadata for Circle3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }
}

impl<T: Scalar> TolerantEq<T> for Circle3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let center_distance = self.center.distance_to(other.center);
        let radius_diff = (self.radius - other.radius).abs();
        let normal_dot = self.axis.as_vector().dot(other.axis.as_vector()).abs();
        let normal_similar = normal_dot >= T::from_f64(0.999);

        center_distance <= tolerance && radius_diff <= tolerance && normal_similar
    }
}

impl<T: Scalar> Circle3D<T> {
    /// 完全な座標系で円を作成（半径が正でない、または参照方向が法線に直交しない場合は `None`）
    fn new_with_ref_direction(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        if axis.dot(ref_direction).abs() > default_angle_tolerance::<T>() {
            return None; // 直交していない
        }

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    /// axisに直交するデフォルトref_directionを計算
    fn compute_default_ref_direction(axis: Direction3D<T>) -> Direction3D<T> {
        // まずX軸との外積を試し、axisがX軸と平行な場合はY軸との外積を使う
        let cross_x = axis.as_vector().cross(Vector3D::unit_x());
        if cross_x.length() > default_kernel_numerical_zero_tolerance::<T>() {
            Direction3D::from_vector(cross_x).unwrap()
        } else {
            Direction3D::from_vector(axis.as_vector().cross(Vector3D::unit_y())).unwrap()
        }
    }
}

impl<T: Scalar> Circle3DConstructor<T> for Circle3D<T> {
    fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        Some(Self {
            center,
            axis,
            ref_direction: Self::compute_default_ref_direction(axis),
            radius,
        })
    }

    fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        <Self as Circle3DConstructor<T>>::new(center, Direction3D::positive_z(), radius)
    }

    fn from_three_points(p1: Point3D<T>, p2: Point3D<T>, p3: Point3D<T>) -> Option<Self> {
        // 3点から平面の法線を計算
        let normal = Vector3D::from_points(p1, p2).cross(Vector3D::from_points(p1, p3));
        if normal.length() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None; // 3点が一直線上
        }
        let axis = Direction3D::from_vector(normal)?;

        // 外心（3 点から等距離の点）: p3 + ((|a|² b − |b|² a) × (a × b)) / (2 |a × b|²)
        let a = Vector3D::from_points(p3, p1);
        let b = Vector3D::from_points(p3, p2);
        let a_cross_b = a.cross(b);
        let numerator = (b * a.length_squared() - a * b.length_squared()).cross(a_cross_b);
        let offset = numerator / ((T::ONE + T::ONE) * a_cross_b.length_squared());
        let center = p3 + offset;

        <Self as Circle3DConstructor<T>>::new(center, axis, center.distance_to(p1))
    }
}

impl<T: Scalar> Circle3DProperties<T> for Circle3D<T> {
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

impl<T: Scalar> Circle3DEvaluation<T> for Circle3D<T> {
    /// `t = 2π` は周期端として `t = 0` と同じ幾何点へ戻る。
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let v_ref = self.ref_direction.as_vector();
        // v_refと垂直なベクトル（円平面内）
        let v_perp = self.axis.as_vector().cross(v_ref);

        self.center + v_ref * (self.radius * t.cos()) + v_perp * (self.radius * t.sin())
    }
}

impl<T: Scalar> Circle3DContainment<T> for Circle3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }

    /// 円周までの距離が既定の許容誤差以内なら `OnBoundary` とする。それ以外は、平面からの距離が
    /// 許容誤差以内で、平面へ投影した点が円の内側にあれば `Inside`、それ以外を `Outside` とする。
    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        let tolerance = default_distance_tolerance::<T>();
        if self.distance_to_point(point) <= tolerance {
            return PointClassification::OnBoundary;
        }

        let to_point = Vector3D::from_points(self.center, point);
        let plane_distance = to_point.dot(self.axis.as_vector());
        if plane_distance.abs() > tolerance {
            return PointClassification::Outside;
        }

        let planar_distance_squared = to_point.length_squared() - plane_distance * plane_distance;
        if planar_distance_squared < self.radius * self.radius {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Circle3DDistance<T> for Circle3D<T> {
    /// 平面からの距離と、平面上での円周までの距離を合成する
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        let to_point = Vector3D::from_points(self.center, point);
        let plane_distance = to_point.dot(self.axis.as_vector()).abs();
        let planar_distance = (to_point.length_squared() - plane_distance * plane_distance)
            .max(T::ZERO)
            .sqrt();
        let radial_distance = (planar_distance - self.radius).abs();

        (plane_distance * plane_distance + radial_distance * radial_distance).sqrt()
    }
}

impl<T: Scalar> Circle3DProjection<T> for Circle3D<T> {
    /// 中心軸上の点は、参照方向の点を最近点とする
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(self.center, point);
        let axis_vec = self.axis.as_vector();
        let in_plane = to_point - axis_vec * to_point.dot(axis_vec);

        let distance = in_plane.length();
        if distance <= default_kernel_numerical_zero_tolerance::<T>() {
            self.center + self.ref_direction.as_vector() * self.radius
        } else {
            self.center + in_plane * (self.radius / distance)
        }
    }
}

impl<T: Scalar> CrossDistance<T, Self> for Circle3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        let center_distance = self.center.distance_to(other.center);
        let radii_sum = self.radius + other.radius;

        if center_distance >= radii_sum {
            center_distance - radii_sum
        } else {
            T::ZERO
        }
    }
}
