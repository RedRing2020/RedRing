//! 境界ボックス（BBox）の新実装
//!
//! foundation.rs の基盤トレイトに基づく BBox3D の実装

use crate::Point3D;
use geo_foundation::{extension_foundation::AbstractBBox, Scalar};

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

    /// 境界ボックスが空かどうかを判定
    /// （無効な境界ボックス、すなわち min > max の場合）
    pub fn is_empty(&self) -> bool {
        self.min.x() > self.max.x() || self.min.y() > self.max.y() || self.min.z() > self.max.z()
    }

    /// 複数の点から境界ボックスを作成
    pub fn from_points(points: &[Point3D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();
        let mut min_z = points[0].z();
        let mut max_z = points[0].z();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        Some(Self::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        ))
    }
}

// ============================================================================
// Default implementation
// ============================================================================

impl<T: Scalar> Default for BBox3D<T> {
    fn default() -> Self {
        // 原点を中心とした単位立方体
        let half = T::ONE / (T::ONE + T::ONE);
        BBox3D::new(
            Point3D::new(-half, -half, -half),
            Point3D::new(half, half, half),
        )
    }
}

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> AbstractBBox<T> for BBox3D<T> {
    type Point = Point3D<T>;

    fn min(&self) -> Self::Point {
        self.min
    }

    fn max(&self) -> Self::Point {
        self.max
    }
}
