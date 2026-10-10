//! InfiniteLine3D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine3D の拡張実装

use crate::{Direction3D, InfiniteLine3D, Point2D, Point3D, Vector2D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> InfiniteLine3D<T> {
    /// X軸に平行な直線を作成
    pub fn x_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_x()).unwrap()
    }

    /// Y軸に平行な直線を作成
    pub fn y_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_y()).unwrap()
    }

    /// Z軸に平行な直線を作成
    pub fn z_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_z()).unwrap()
    }

    /// 軸に平行かどうかを判定
    pub fn is_parallel_to_axis(&self, axis: Vector3D<T>, tolerance: T) -> bool {
        let normalized_axis =
            Direction3D::from_vector(axis.normalize()).unwrap_or(Direction3D::positive_x());
        let dot_product = self.direction_internal().dot(&normalized_axis).abs();
        (dot_product - T::ONE).abs() <= tolerance
    }

    /// 方向ベクトルの外積（内部ヘルパー）
    fn cross_product_of_directions(&self, other: &Self) -> Vector3D<T> {
        self.direction_internal()
            .as_vector()
            .cross(&other.direction_internal().as_vector())
    }

    /// 直線が平行かを判定
    pub fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        self.cross_product_of_directions(other).length() <= tolerance
    }

    /// 直線が同一かを判定
    pub fn is_coincident(&self, other: &Self, tolerance: T) -> bool {
        self.is_parallel(other, tolerance)
            && self.contains_point(&other.point_internal(), tolerance)
    }

    /// 直線を平行移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        Self::new(
            Point3D::new(
                self.point_internal().x() + offset.x(),
                self.point_internal().y() + offset.y(),
                self.point_internal().z() + offset.z(),
            ),
            self.direction_internal().as_vector(),
        )
        .unwrap()
    }

    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self::new(
            self.point_internal(),
            (-self.direction_internal()).as_vector(),
        )
        .unwrap()
    }

    /// 指定点周りの回転（軸と角度指定）
    pub fn rotate_around_point(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: T,
    ) -> Option<Self> {
        let normalized_axis = axis.normalize();
        if normalized_axis.length() <= T::EPSILON {
            return None;
        }

        // Rodriguesの回転公式を使用
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 点の回転
        let to_point = Vector3D::new(
            self.point_internal().x() - center.x(),
            self.point_internal().y() - center.y(),
            self.point_internal().z() - center.z(),
        );

        let term1 = to_point * cos_angle;
        let term2 = normalized_axis.cross(&to_point) * sin_angle;
        let term3 = normalized_axis * (normalized_axis.dot(&to_point) * (T::ONE - cos_angle));
        let rotated_to_point = Vector3D::new(
            term1.x() + term2.x() + term3.x(),
            term1.y() + term2.y() + term3.y(),
            term1.z() + term2.z() + term3.z(),
        );

        let new_point = Point3D::new(
            center.x() + rotated_to_point.x(),
            center.y() + rotated_to_point.y(),
            center.z() + rotated_to_point.z(),
        );

        // 方向ベクトルの回転
        let rotated_direction = self.direction_internal() * cos_angle
            + normalized_axis.cross(&self.direction_internal()) * sin_angle
            + normalized_axis
                * (normalized_axis.dot(&self.direction_internal()) * (T::ONE - cos_angle));

        Some(Self::new(new_point, rotated_direction).unwrap())
    }

    /// 原点周りの回転
    pub fn rotate_around_origin(&self, axis: Vector3D<T>, angle: T) -> Option<Self> {
        self.rotate_around_point(Point3D::origin(), axis, angle)
    }

    /// 2次元投影（Z成分を無視）
    pub fn to_2d(&self) -> crate::InfiniteLine2D<T> {
        let point_2d = Point2D::new(self.point_internal().x(), self.point_internal().y());
        let direction_2d =
            Vector2D::new(self.direction_internal().x(), self.direction_internal().y());
        crate::InfiniteLine2D::new(point_2d, direction_2d).unwrap()
    }
}
