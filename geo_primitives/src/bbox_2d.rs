//! 2次元境界ボックス（BBox2D）の新実装
//!
//! foundation.rs の基盤トレイトに基づく BBox2D の実装

use crate::Point2D;
use geo_foundation::Scalar;

/// 2次元軸平行境界ボックス
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox2D<T: Scalar> {
    min: Point2D<T>,
    max: Point2D<T>,
}

impl<T: Scalar> BBox2D<T> {
    /// 新しい境界ボックスを作成
    pub fn new(min: Point2D<T>, max: Point2D<T>) -> Self {
        Self { min, max }
    }

    /// 点から境界ボックスを作成（点の境界ボックス = 点自身）
    pub fn from_point(point: Point2D<T>) -> Self {
        Self::new(point, point)
    }

    /// 複数の点から境界ボックスを作成
    pub fn from_points(points: &[Point2D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
        }

        Some(Self::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    /// 最小点を取得
    pub fn min(&self) -> Point2D<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max(&self) -> Point2D<T> {
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

    /// 境界ボックスの面積を取得
    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    /// 境界ボックスの中心点を取得
    pub fn center(&self) -> Point2D<T> {
        let two = T::ONE + T::ONE;
        Point2D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    /// 境界ボックスのサイズ（幅、高さ）を取得
    pub fn size(&self) -> (T, T) {
        (self.width(), self.height())
    }

    /// 点が境界ボックス内に含まれるかを判定
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// 他の境界ボックスが完全に含まれるかを判定
    pub fn contains_bbox(&self, other: &Self) -> bool {
        self.contains_point(&other.min) && self.contains_point(&other.max)
    }

    /// 他の境界ボックスと交差するかを判定
    pub fn intersects(&self, other: &Self) -> bool {
        !(self.max.x() < other.min.x()
            || other.max.x() < self.min.x()
            || self.max.y() < other.min.y()
            || other.max.y() < self.min.y())
    }

    /// 他の境界ボックスとの交差領域を取得
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let min_x = self.min.x().max(other.min.x());
        let min_y = self.min.y().max(other.min.y());
        let max_x = self.max.x().min(other.max.x());
        let max_y = self.max.y().min(other.max.y());

        Some(Self::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    /// 他の境界ボックスとの結合領域を取得
    pub fn union(&self, other: &Self) -> Self {
        let min_x = self.min.x().min(other.min.x());
        let min_y = self.min.y().min(other.min.y());
        let max_x = self.max.x().max(other.max.x());
        let max_y = self.max.y().max(other.max.y());

        Self::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 境界ボックスを指定マージンで拡張
    pub fn expand(&self, margin: T) -> Self {
        Self::new(
            Point2D::new(self.min.x() - margin, self.min.y() - margin),
            Point2D::new(self.max.x() + margin, self.max.y() + margin),
        )
    }

    /// 境界ボックスが退化しているか（面積が0）
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.width() <= tolerance || self.height() <= tolerance
    }

    /// 3次元境界ボックスに拡張（Z=0）
    pub fn to_3d(&self) -> crate::BBox3D<T> {
        crate::BBox3D::new(self.min.to_3d(), self.max.to_3d())
    }

    /// 3次元境界ボックスに拡張（指定Z範囲）
    pub fn to_3d_with_z(&self, min_z: T, max_z: T) -> crate::BBox3D<T> {
        crate::BBox3D::new(self.min.to_3d_with_z(min_z), self.max.to_3d_with_z(max_z))
    }
}
