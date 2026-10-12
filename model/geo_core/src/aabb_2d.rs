//! 2次元軸平行境界ボックス（Aabb2D）
//!
//! Foundation Pattern に基づく Aabb2D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

#[cfg(test)]
mod tests;

use crate::Point2D;
use crate::PointClassification;
use analysis::abstract_types::Scalar;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2D<T: Scalar> {
    min: Point2D<T>,
    max: Point2D<T>,
}

impl<T: Scalar> Aabb2D<T> {
    /// 新しいAABBを作成
    pub fn new(min: Point2D<T>, max: Point2D<T>) -> Self {
        Self { min, max }
    }

    /// 複数の点からAABBを作成
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
    pub fn min_point(&self) -> Point2D<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max_point(&self) -> Point2D<T> {
        self.max
    }

    /// 幅
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 高さ
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 面積
    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    /// 中心点
    pub fn center(&self) -> Point2D<T> {
        let two = T::ONE + T::ONE;
        Point2D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    /// 点がAABB内に含まれるか
    pub fn contains_point(&self, point: Point2D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// 点から AABB の境界（辺・面）までの最短距離
    ///
    /// 内部の点は最も近い辺・面までの距離、外部の点は AABB までの距離を返す。
    pub fn distance_to_boundary(&self, point: Point2D<T>) -> T {
        let coordinates = [point.x(), point.y()];
        let mins = [self.min.x(), self.min.y()];
        let maxs = [self.max.x(), self.max.y()];

        let axes = coordinates.iter().zip(mins.iter().zip(maxs.iter()));
        if self.contains_point(point) {
            axes.map(|(&value, (&min, &max))| (value - min).min(max - value))
                .fold(maxs[0] - mins[0], |nearest, margin| nearest.min(margin))
        } else {
            axes.map(|(&value, (&min, &max))| {
                let excess = (min - value).max(value - max).max(T::ZERO);
                excess * excess
            })
            .fold(T::ZERO, |sum, squared| sum + squared)
            .sqrt()
        }
    }

    /// AABB の領域に対する点の位置を分類する
    ///
    /// 境界までの距離が `tolerance` 以内なら `OnBoundary`、それ以外は内部・外部に分ける。
    /// `contains_point` は `classify_point(point, 0)` が `Outside` でないことと一致する。
    pub fn classify_point(&self, point: Point2D<T>, tolerance: T) -> PointClassification {
        if self.distance_to_boundary(point) <= tolerance {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }

    /// 他のAABBと交差するか
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
    }

    /// 他のAABBを完全に含むか
    pub fn contains_aabb(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.max.x() >= other.max.x()
            && self.min.y() <= other.min.y()
            && self.max.y() >= other.max.y()
    }

    /// is_empty
    pub fn is_empty(&self) -> bool {
        self.min.x() > self.max.x() || self.min.y() > self.max.y()
    }
}
