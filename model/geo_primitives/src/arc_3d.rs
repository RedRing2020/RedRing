//! 3次元円弧（Arc3D）の実装
//!
//! 中心・半径・法線・開始方向のフレームと、開始角から反時計回りに終了角までの角度範囲で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Angle, Direction3D, Point3D, Vector3D};
use analysis::AngleRange;
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, default_kernel_numerical_zero_tolerance,
    Arc3DConstructor, Arc3DContainment, Arc3DDerived, Arc3DDistance, Arc3DEndpoint,
    Arc3DEvaluation, Arc3DProperties, Arc3DSampling, Arc3DTrimRange, PrimitiveKind,
    PrimitiveMetadata, Scalar, TolerantEq,
};

/// 3次元円弧
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc3D<T: Scalar> {
    center: Point3D<T>,
    radius: T,
    normal: Direction3D<T>,    // 円弧平面の法線ベクトル（正規化済み）
    start_dir: Direction3D<T>, // 開始方向ベクトル（正規化済み）
    start_angle: Angle<T>,     // 開始角度
    end_angle: Angle<T>,       // 終了角度
}

impl<T: Scalar> PrimitiveMetadata for Arc3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> TolerantEq<T> for Arc3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let center_distance = self.center.distance_to(other.center);
        let radius_diff = (self.radius - other.radius).abs();
        let start_angle_diff = (self.start_angle - other.start_angle).to_radians().abs();
        let end_angle_diff = (self.end_angle - other.end_angle).to_radians().abs();
        let tolerance_angle = T::from_f64(0.01);

        center_distance <= tolerance
            && radius_diff <= tolerance
            && start_angle_diff <= tolerance_angle
            && end_angle_diff <= tolerance_angle
    }
}

impl<T: Scalar> Arc3D<T> {
    /// 法線に垂直なベクトルを計算
    fn compute_perpendicular(normal: Direction3D<T>) -> Option<Direction3D<T>> {
        let n = normal.as_vector();
        let perp_x = n.cross(Vector3D::unit_x());
        if perp_x.length() > default_kernel_numerical_zero_tolerance::<T>() {
            Direction3D::from_vector(perp_x)
        } else {
            // X軸と平行な場合、Y軸を使用
            Direction3D::from_vector(n.cross(Vector3D::unit_y()))
        }
    }

    /// 3点の外心（3点を通る円の中心）を計算
    ///
    /// `a = p1 - p3`、`b = p2 - p3` として
    /// `p3 + ((|a|² b - |b|² a) × (a × b)) / (2 |a × b|²)`。
    fn calculate_circle_center(
        p1: Point3D<T>,
        p2: Point3D<T>,
        p3: Point3D<T>,
    ) -> Option<Point3D<T>> {
        let a = Vector3D::from_points(p3, p1);
        let b = Vector3D::from_points(p3, p2);
        let a_cross_b = a.cross(b);
        let denominator = (T::ONE + T::ONE) * a_cross_b.length_squared();
        if denominator <= default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }
        let numerator = (b * a.length_squared() - a * b.length_squared()).cross(a_cross_b);
        Some(p3 + numerator * (T::ONE / denominator))
    }

    /// `dir1` から `dir2` まで、法線まわりに反時計回り（右手系の正の向き）に測った角度
    ///
    /// 3点から作る円弧は 180° を超えうるため、`acos` ではなく符号付きの角度で
    /// (0, 2π] の範囲に求める。
    fn calculate_angle_between(
        dir1: Direction3D<T>,
        dir2: Direction3D<T>,
        normal: Direction3D<T>,
    ) -> Angle<T> {
        let u = dir1.as_vector();
        let v = dir2.as_vector();
        let sin = normal.as_vector().dot(u.cross(v));
        let cos = u.dot(v);
        let mut angle = sin.atan2(cos);
        if angle <= T::ZERO {
            angle += T::TAU;
        }
        Angle::from_radians(angle)
    }

    /// 開始角から反時計回りに終了角まで進む角度範囲（開始角と終了角が等しい場合は `None`）
    fn angle_range(&self) -> Option<AngleRange<T>> {
        AngleRange::from_ccw_bounds(self.start_angle, self.end_angle)
    }
}

impl<T: Scalar> Arc3DConstructor<T> for Arc3D<T> {
    fn new(
        center: Point3D<T>,
        radius: T,
        normal: Direction3D<T>,
        start_dir: Direction3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // 法線と開始方向の直交性チェック
        if normal.dot(start_dir).abs() > default_angle_tolerance::<T>() {
            return None;
        }

        Some(Self {
            center,
            radius,
            normal,
            start_dir,
            start_angle,
            end_angle,
        })
    }

    fn xy_arc(
        center: Point3D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        <Self as Arc3DConstructor<T>>::new(
            center,
            radius,
            Direction3D::positive_z(),
            Direction3D::positive_x(),
            start_angle,
            end_angle,
        )
    }

    fn from_three_points(start: Point3D<T>, mid: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        // 3点が同一直線上でないことを確認
        let cross = Vector3D::from_points(start, mid).cross(Vector3D::from_points(mid, end));
        if cross.length() < default_distance_tolerance::<T>() {
            return None;
        }

        let center = Self::calculate_circle_center(start, mid, end)?;
        let normal = Direction3D::from_vector(cross)?;
        let start_dir = Direction3D::from_vector(Vector3D::from_points(center, start))?;
        let end_dir = Direction3D::from_vector(Vector3D::from_points(center, end))?;

        // 開始点 → 中間点 → 終了点の順に、法線まわりに反時計回りとなる
        let end_angle = Self::calculate_angle_between(start_dir, end_dir, normal);

        <Self as Arc3DConstructor<T>>::new(
            center,
            center.distance_to(start),
            normal,
            start_dir,
            Angle::from_radians(T::ZERO),
            end_angle,
        )
    }

    fn full_circle(center: Point3D<T>, normal: Direction3D<T>, radius: T) -> Option<Self> {
        <Self as Arc3DConstructor<T>>::new(
            center,
            radius,
            normal,
            Self::compute_perpendicular(normal)?,
            Angle::from_radians(T::ZERO),
            Angle::from_radians(T::TAU),
        )
    }
}

impl<T: Scalar> Arc3DProperties<T> for Arc3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    fn start_direction(&self) -> Direction3D<T> {
        self.start_dir
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    fn angle_span(&self) -> Angle<T> {
        let span = self.end_angle.to_radians() - self.start_angle.to_radians();
        if span < T::ZERO {
            Angle::from_radians(span + T::TAU)
        } else {
            Angle::from_radians(span)
        }
    }

    fn is_full_circle(&self) -> bool {
        (self.angle_span().to_radians() - T::TAU).abs() <= default_angle_tolerance::<T>()
    }
}

impl<T: Scalar> Arc3DDerived<T> for Arc3D<T> {
    fn length(&self) -> T {
        self.radius * self.angle_span().to_radians()
    }
}

impl<T: Scalar> Arc3DEndpoint<T> for Arc3D<T> {
    fn start_point(&self) -> Point3D<T> {
        self.point_at_angle(self.start_angle)
    }

    fn end_point(&self) -> Point3D<T> {
        self.point_at_angle(self.end_angle)
    }
}

impl<T: Scalar> Arc3DEvaluation<T> for Arc3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        self.point_at_angle(Angle::from_radians(
            self.start_angle.to_radians() + self.angle_span().to_radians() * t,
        ))
    }

    /// 開始方向を角度 0 とし、法線まわりに反時計回りを正とする
    fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        let radians = angle.to_radians();
        let u_axis = self.start_dir.as_vector();
        let v_axis = self.normal.as_vector().cross(u_axis);

        self.center
            + u_axis * (self.radius * radians.cos())
            + v_axis * (self.radius * radians.sin())
    }
}

impl<T: Scalar> Arc3DDistance<T> for Arc3D<T> {
    /// 点の角度が角度範囲内なら母円までの距離（円弧平面からの距離を含む）、範囲外なら始点・終点までの
    /// 距離の小さい方を返す。
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        if self.contains_point_angle(point) {
            let to_point = Vector3D::from_points(self.center, point);
            let normal = self.normal.as_vector();
            let plane_distance = to_point.dot(normal);
            let radial_distance = (to_point - normal * plane_distance).length() - self.radius;
            (plane_distance * plane_distance + radial_distance * radial_distance).sqrt()
        } else {
            point
                .distance_to(self.start_point())
                .min(point.distance_to(self.end_point()))
        }
    }
}

impl<T: Scalar> Arc3DContainment<T> for Arc3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Arc3DTrimRange<T> for Arc3D<T> {
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let tolerance = default_angle_tolerance::<T>();
        match self.angle_range() {
            Some(range) => range.contains(angle, tolerance),
            None => angle.is_equivalent(&self.start_angle, tolerance),
        }
    }

    fn contains_point_angle(&self, point: Point3D<T>) -> bool {
        let to_point = Vector3D::from_points(self.center, point);
        let u_axis = self.start_dir.as_vector();
        let v_axis = self.normal.as_vector().cross(u_axis);
        let angle = to_point.dot(v_axis).atan2(to_point.dot(u_axis));
        self.contains_angle(Angle::from_radians(angle))
    }
}

impl<T: Scalar> Arc3DSampling<T> for Arc3D<T> {
    fn sample_points(&self, num_points: usize) -> Vec<Point3D<T>> {
        match num_points {
            0 => Vec::new(),
            1 => vec![self.start_point()],
            _ => {
                let segments = T::from_usize(num_points - 1);
                (0..num_points)
                    .map(|i| self.point_at_parameter(T::from_usize(i) / segments))
                    .collect()
            }
        }
    }
}
