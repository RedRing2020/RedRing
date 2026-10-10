//! InfiniteLine3D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine3D の拡張実装

use crate::{InfiniteLine3D, Point3D, Vector3D};
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
}
