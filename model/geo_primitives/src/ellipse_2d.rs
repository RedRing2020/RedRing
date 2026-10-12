//! 2次元楕円（Ellipse2D）の実装
//!
//! 中心・長半軸・短半軸・回転角（X 軸から長軸までの角度）で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Angle, Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, Ellipse2DConstructor, Ellipse2DContainment, Ellipse2DDistance,
    Ellipse2DEvaluation, Ellipse2DProjection, Ellipse2DProperties, PointClassification,
    PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 2次元楕円
///
/// 長軸・短軸を持つ楕円を表現
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse2D<T: Scalar> {
    center: Point2D<T>, // 中心点
    semi_major: T,      // 長半軸（a）
    semi_minor: T,      // 短半軸（b）
    rotation: Angle<T>, // 回転角（X軸から長軸まで）
}

impl<T: Scalar> PrimitiveMetadata for Ellipse2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }
}

impl<T: Scalar> Ellipse2D<T> {
    /// 点に最も近い楕円上の点の local angle parameter（`0 <= t < 2π`）を取得
    ///
    /// 楕円弧が点のパラメータを求めるため pub(crate)。`point_at_parameter(t)` が点の最近点となる `t` を返す。
    pub(crate) fn parameter_for_point(&self, point: Point2D<T>) -> T {
        let (x_local, y_local) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major,
            self.semi_minor,
        );
        let angle = (closest_y / self.semi_minor).atan2(closest_x / self.semi_major);
        if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        }
    }

    /// 楕円の周回長の近似（ラマヌジャンの公式）
    ///
    /// 楕円弧の長さに使うため pub(crate)。
    pub(crate) fn circumference(&self) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));

        let three = T::ONE + T::ONE + T::ONE;
        let ten = three + three + three + T::ONE;
        let four = T::ONE + T::ONE + T::ONE + T::ONE;

        T::PI * (a + b) * (T::ONE + (three * h) / (ten + (four - three * h).sqrt()))
    }

    /// 長軸方向の単位ベクトル
    fn major_axis_direction(&self) -> Vector2D<T> {
        let rotation = self.rotation.to_radians();
        Vector2D::new(rotation.cos(), rotation.sin())
    }

    /// 中心を原点とし、長軸を x 軸とする楕円の局所座標系での点の座標
    fn local_coordinates(&self, point: Point2D<T>) -> (T, T) {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let rotation = self.rotation.to_radians();
        let (cos_theta, sin_theta) = (rotation.cos(), rotation.sin());

        (
            dx * cos_theta + dy * sin_theta,
            -dx * sin_theta + dy * cos_theta,
        )
    }

    /// 局所座標系の点を全体座標系へ戻す
    fn local_to_world(&self, x_local: T, y_local: T) -> Point2D<T> {
        let rotation = self.rotation.to_radians();
        let (cos_theta, sin_theta) = (rotation.cos(), rotation.sin());
        Point2D::new(
            self.center.x() + x_local * cos_theta - y_local * sin_theta,
            self.center.y() + x_local * sin_theta + y_local * cos_theta,
        )
    }
}

impl<T: Scalar> Ellipse2DConstructor<T> for Ellipse2D<T> {
    fn new(center: Point2D<T>, semi_major: T, semi_minor: T, rotation: Angle<T>) -> Option<Self> {
        (semi_major > T::ZERO && semi_minor > T::ZERO && semi_major >= semi_minor).then_some(Self {
            center,
            semi_major,
            semi_minor,
            rotation,
        })
    }
}

impl<T: Scalar> Ellipse2DProperties<T> for Ellipse2D<T> {
    fn center(&self) -> Point2D<T> {
        self.center
    }

    fn semi_major_axis(&self) -> T {
        self.semi_major
    }

    fn semi_minor_axis(&self) -> T {
        self.semi_minor
    }

    fn rotation(&self) -> Angle<T> {
        self.rotation
    }
}

impl<T: Scalar> Ellipse2DEvaluation<T> for Ellipse2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.local_to_world(self.semi_major * t.cos(), self.semi_minor * t.sin())
    }
}

impl<T: Scalar> Ellipse2DContainment<T> for Ellipse2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }

    /// 楕円までの距離が既定の許容誤差以内なら `OnBoundary`、それ以外は楕円の方程式で内側・外側に分ける。
    fn classify_point(&self, point: Point2D<T>) -> PointClassification {
        if self.contains_point(point) {
            return PointClassification::OnBoundary;
        }

        let (x_local, y_local) = self.local_coordinates(point);
        let x_normalized = x_local / self.semi_major;
        let y_normalized = y_local / self.semi_minor;
        if x_normalized * x_normalized + y_normalized * y_normalized < T::ONE {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Ellipse2DDistance<T> for Ellipse2D<T> {
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        let (x_local, y_local) = self.local_coordinates(point);
        geo_commons::ellipse_2d_distance_to_point(
            x_local,
            y_local,
            self.semi_major,
            self.semi_minor,
        )
    }
}

impl<T: Scalar> Ellipse2DProjection<T> for Ellipse2D<T> {
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T> {
        let (x_local, y_local) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major,
            self.semi_minor,
        );
        self.local_to_world(closest_x, closest_y)
    }
}
