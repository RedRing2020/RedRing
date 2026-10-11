//! 2次元楕円（Ellipse2D）の新実装
//!
//! 新しいtraitsシステムに対応したEllipse2Dの実装

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, Ellipse2DConstructor, Ellipse2DContainment, Ellipse2DDistance,
    Ellipse2DEvaluation, Ellipse2DProjection, Ellipse2DProperties, PointClassification,
    PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 2次元楕円
///
/// 長軸・短軸を持つ楕円を表現
/// 円は半径が等しい特殊な楕円として扱える
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse2D<T: Scalar> {
    center: Point2D<T>, // 中心点
    semi_major: T,      // 長半軸（a）
    semi_minor: T,      // 短半軸（b）
    rotation: T,        // 回転角（ラジアン、X軸からの回転）
}

impl<T: Scalar> PrimitiveMetadata for Ellipse2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }
}

impl<T: Scalar> Ellipse2D<T> {
    /// 新しい楕円を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `semi_major` - 長半軸（a >= b である必要がある）
    /// * `semi_minor` - 短半軸
    /// * `rotation` - 回転角（ラジアン）
    pub fn new(center: Point2D<T>, semi_major: T, semi_minor: T, rotation: T) -> Option<Self> {
        if semi_major > T::ZERO && semi_minor > T::ZERO && semi_major >= semi_minor {
            Some(Self {
                center,
                semi_major,
                semi_minor,
                rotation,
            })
        } else {
            None
        }
    }

    /// 回転角を取得（ラジアン）
    pub fn rotation(&self) -> T {
        self.rotation
    }

    /// 面積を取得
    pub fn area(&self) -> T {
        T::PI * self.semi_major * self.semi_minor
    }

    /// 周回長を取得（ラマヌジャンの近似式）
    pub fn circumference(&self) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));

        // ラマヌジャンの第二近似式の簡易版
        let three = T::ONE + T::ONE + T::ONE;
        let ten = three + three + three + T::ONE;
        let four = T::ONE + T::ONE + T::ONE + T::ONE;

        T::PI * (a + b) * (T::ONE + (three * h) / (ten + (four - three * h).sqrt()))
    }

    /// 点が楕円上にあるか判定（楕円までの距離が `tolerance` 以内）
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 楕円が囲む領域に対する点の位置を分類する
    ///
    /// 楕円までの距離が `tolerance` 以内なら `OnBoundary`、それ以外は楕円の方程式で内側・外側に分ける。
    pub fn classify_point(&self, point: &Point2D<T>, tolerance: T) -> PointClassification {
        if self.contains_point(point, tolerance) {
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

    /// 点から楕円（曲線）への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let (x_local, y_local) = self.local_coordinates(point);
        geo_commons::ellipse_2d_distance_to_point(
            x_local,
            y_local,
            self.semi_major,
            self.semi_minor,
        )
    }

    /// 点に最も近い楕円上の点を取得
    pub fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        let (x_local, y_local) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major,
            self.semi_minor,
        );

        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();
        Point2D::new(
            self.center.x() + closest_x * cos_theta - closest_y * sin_theta,
            self.center.y() + closest_x * sin_theta + closest_y * cos_theta,
        )
    }

    /// 点に最も近い楕円上の点の local angle parameter（`0 <= t < 2π`）を取得
    ///
    /// `point_at_parameter(t)` が点の最近点となる `t` を返す。
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
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

    /// 中心を原点とし、長軸を x 軸とする楕円の局所座標系での点の座標
    fn local_coordinates(&self, point: &Point2D<T>) -> (T, T) {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        (
            dx * cos_theta + dy * sin_theta,
            -dx * sin_theta + dy * cos_theta,
        )
    }

    /// Primitive 局所座標系の angle parameter `t` (`0 <= t < 2π`) で楕円上の点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円上の点（回転前）
        let x_local = self.semi_major * cos_t;
        let y_local = self.semi_minor * sin_t;

        // 回転を適用
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let x_rotated = x_local * cos_theta - y_local * sin_theta;
        let y_rotated = x_local * sin_theta + y_local * cos_theta;

        // 中心を考慮した最終座標
        Point2D::new(self.center.x() + x_rotated, self.center.y() + y_rotated)
    }

    /// `point_at_parameter` と同じ local angle domain における接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円の接線ベクトル（回転前）
        let dx_local = -self.semi_major * sin_t;
        let dy_local = self.semi_minor * cos_t;

        // 回転を適用
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let dx_rotated = dx_local * cos_theta - dy_local * sin_theta;
        let dy_rotated = dx_local * sin_theta + dy_local * cos_theta;

        Vector2D::new(dx_rotated, dy_rotated)
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        // 回転を考慮した楕円の境界ボックス計算
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let a = self.semi_major;
        let b = self.semi_minor;

        // 回転楕円の幅と高さ
        let width = ((a * cos_theta) * (a * cos_theta) + (b * sin_theta) * (b * sin_theta)).sqrt();
        let height = ((a * sin_theta) * (a * sin_theta) + (b * cos_theta) * (b * cos_theta)).sqrt();

        geo_core::Aabb2D::new(
            Point2D::new(self.center.x() - width, self.center.y() - height),
            Point2D::new(self.center.x() + width, self.center.y() + height),
        )
    }
}

impl<T: Scalar> Ellipse2D<T> {
    /// local angle parameter の有効範囲 `[0, 2π]` を返す
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU) // 0 から 2π
    }

    /// 長軸方向の単位ベクトルを取得
    pub fn major_axis_direction(&self) -> Vector2D<T> {
        Vector2D::new(self.rotation.cos(), self.rotation.sin())
    }

    /// 短軸方向の単位ベクトルを取得
    pub fn minor_axis_direction(&self) -> Vector2D<T> {
        Vector2D::new(-self.rotation.sin(), self.rotation.cos())
    }
}

impl<T: Scalar> Ellipse2DConstructor<T> for Ellipse2D<T> {
    fn new(center: (T, T), semi_major: T, semi_minor: T, rotation: T) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        Self::new(center_point, semi_major, semi_minor, rotation)
    }
}

impl<T: Scalar> Ellipse2DProperties<T> for Ellipse2D<T> {
    fn center(&self) -> (T, T) {
        (self.center.x(), self.center.y())
    }

    fn semi_major_axis(&self) -> T {
        self.semi_major
    }

    fn semi_minor_axis(&self) -> T {
        self.semi_minor
    }

    fn rotation(&self) -> T {
        self.rotation
    }
}

impl<T: Scalar + From<f64>> Ellipse2DEvaluation<T> for Ellipse2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let point = Ellipse2D::point_at_parameter(self, t);
        (point.x(), point.y())
    }
}

impl<T: Scalar + From<f64>> Ellipse2DContainment<T> for Ellipse2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        let tolerance = default_distance_tolerance::<T>();
        Ellipse2D::contains_point(self, &p, tolerance)
    }

    fn classify_point(&self, point: (T, T)) -> PointClassification {
        let p = Point2D::new(point.0, point.1);
        Ellipse2D::classify_point(self, &p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar + From<f64>> Ellipse2DDistance<T> for Ellipse2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        Ellipse2D::distance_to_point(self, &p)
    }
}

impl<T: Scalar + From<f64>> Ellipse2DProjection<T> for Ellipse2D<T> {
    fn closest_point(&self, point: (T, T)) -> (T, T) {
        let p = Point2D::new(point.0, point.1);
        let closest = Ellipse2D::closest_point(self, &p);
        (closest.x(), closest.y())
    }
}
