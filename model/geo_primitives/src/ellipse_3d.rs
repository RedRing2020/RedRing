//! 3次元楕円（Ellipse3D）の実装
//!
//! 中心・長半軸・短半軸・平面の法線・長軸方向で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Angle, Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, Ellipse3DConstructor,
    Ellipse3DContainment, Ellipse3DDistance, Ellipse3DEvaluation, Ellipse3DProjection,
    Ellipse3DProperties, PointClassification, PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 3次元楕円
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Direction3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Direction3D<T>, // 長軸方向ベクトル（正規化済み）
}

impl<T: Scalar> PrimitiveMetadata for Ellipse3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }
}

impl<T: Scalar> Ellipse3D<T> {
    /// 楕円の周回長の近似（ラマヌジャンの公式）
    ///
    /// 楕円弧の長さに使うため pub(crate)。
    pub(crate) fn circumference(&self) -> T {
        let a = self.semi_major_axis;
        let b = self.semi_minor_axis;
        let h = ((a - b) / (a + b)).powi(2);

        T::PI
            * (a + b)
            * (T::ONE
                + (T::from_f64(3.0) * h)
                    / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
    }

    /// 点に最も近い楕円上の点の local angle parameter（`0 <= t < 2π`）を取得
    ///
    /// 楕円弧が点のパラメータを求めるため pub(crate)。平面外の点は平面へ投影した点の最近点を使う。
    pub(crate) fn parameter_for_point(&self, point: Point3D<T>) -> T {
        let (x_local, y_local, _) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        );
        let angle = (closest_y / self.semi_minor_axis).atan2(closest_x / self.semi_major_axis);
        if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        }
    }

    /// 中心を原点とし、長軸・短軸・法線を軸とする楕円の局所座標系での点の座標
    fn local_coordinates(&self, point: Point3D<T>) -> (T, T, T) {
        let translated = Vector3D::from_points(self.center, point);
        (
            translated.dot(self.major_axis_dir.as_vector()),
            translated.dot(self.minor_axis_direction().as_vector()),
            translated.dot(self.normal.as_vector()),
        )
    }

    /// 平面上の局所座標（長軸・短軸）の点を全体座標系へ戻す
    fn local_to_world(&self, x_local: T, y_local: T) -> Point3D<T> {
        self.center
            + self.major_axis_dir.as_vector() * x_local
            + self.minor_axis_direction().as_vector() * y_local
    }
}

impl<T: Scalar> Ellipse3DConstructor<T> for Ellipse3D<T> {
    fn new(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        normal: Direction3D<T>,
        major_axis_direction: Direction3D<T>,
    ) -> Option<Self> {
        if semi_major_axis < semi_minor_axis || semi_minor_axis <= T::ZERO {
            return None;
        }

        // 長軸方向は法線に直交する
        if normal.dot(major_axis_direction).abs() > default_angle_tolerance::<T>() {
            return None;
        }

        Some(Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            normal,
            major_axis_dir: major_axis_direction,
        })
    }

    fn new_xy_plane(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: Angle<T>,
    ) -> Option<Self> {
        let radians = rotation.to_radians();
        let major_dir = Direction3D::new(radians.cos(), radians.sin(), T::ZERO)?;
        <Self as Ellipse3DConstructor<T>>::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            Direction3D::positive_z(),
            major_dir,
        )
    }
}

impl<T: Scalar> Ellipse3DProperties<T> for Ellipse3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center
    }

    fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    fn major_axis_direction(&self) -> Direction3D<T> {
        self.major_axis_dir
    }

    fn minor_axis_direction(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.normal.cross(*self.major_axis_dir))
            .expect("Cross product of orthogonal unit vectors should be valid")
    }

    fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }
}

impl<T: Scalar> Ellipse3DEvaluation<T> for Ellipse3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        self.local_to_world(
            self.semi_major_axis * t.cos(),
            self.semi_minor_axis * t.sin(),
        )
    }
}

impl<T: Scalar> Ellipse3DContainment<T> for Ellipse3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }

    /// 楕円までの距離が既定の許容誤差以内なら `OnBoundary` とする。それ以外は、平面からの距離が
    /// 許容誤差以内で、平面へ投影した点が楕円の内側にあれば `Inside`、それ以外を `Outside` とする。
    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        if self.contains_point(point) {
            return PointClassification::OnBoundary;
        }

        let (x_local, y_local, z_local) = self.local_coordinates(point);
        if z_local.abs() > default_distance_tolerance::<T>() {
            return PointClassification::Outside;
        }

        let x_normalized = x_local / self.semi_major_axis;
        let y_normalized = y_local / self.semi_minor_axis;
        if x_normalized * x_normalized + y_normalized * y_normalized < T::ONE {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Ellipse3DDistance<T> for Ellipse3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        let (x_local, y_local, z_local) = self.local_coordinates(point);
        geo_commons::ellipse_3d_distance_to_point(
            x_local,
            y_local,
            z_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }
}

impl<T: Scalar> Ellipse3DProjection<T> for Ellipse3D<T> {
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T> {
        let (x_local, y_local, _) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        );
        self.local_to_world(closest_x, closest_y)
    }
}
