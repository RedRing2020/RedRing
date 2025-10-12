//! 境界ボックス（BBox）の新実装
//!
//! foundation.rs の基盤トレイトに基づく BBox3D の実装

use crate::Point3D;
use geo_foundation::Scalar;

/// 3次元軸平行境界ボックス
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox3D<T: Scalar> {
    min: Point3D<T>,
    max: Point3D<T>,
}

impl<T: Scalar> BBox3D<T> {
    /// 新しい境界ボックスを作成
    pub fn new(min: Point3D<T>, max: Point3D<T>) -> Self {
        Self { min, max }
    }

    /// 点から境界ボックスを作成（点の境界ボックス = 点自身）
    pub fn from_point(point: Point3D<T>) -> Self {
        Self::new(point, point)
    }

    /// 最小点を取得
    pub fn min(&self) -> Point3D<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max(&self) -> Point3D<T> {
        self.max
    }

    /// 境界ボックスの幅を取得
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 境界ボックスの高さを取得
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 境界ボックスの深さを取得
    pub fn depth(&self) -> T {
        self.max.z() - self.min.z()
    }

    /// 境界ボックスの中心点を取得
    pub fn center(&self) -> Point3D<T> {
        let two = T::ONE + T::ONE;
        Point3D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
            (self.min.z() + self.max.z()) / two,
        )
    }

    /// 点が境界ボックス内に含まれるかを判定
    pub fn contains_point(&self, point: &Point3D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
            && point.z() >= self.min.z()
            && point.z() <= self.max.z()
    }
}
