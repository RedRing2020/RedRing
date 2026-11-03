//! 3次元軸平行境界ボックス（Aabb3D）
//!
//! Foundation Pattern に基づく Aabb3D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

#[cfg(test)]
mod tests;

use crate::PointClassification;
use analysis::abstract_types::Scalar;

use crate::Point3D;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3D<T: Scalar> {
    min: Point3D<T>,
    max: Point3D<T>,
}

impl<T: Scalar> Aabb3D<T> {
    /// 新しいAABBを作成
    pub fn new(min: Point3D<T>, max: Point3D<T>) -> Self {
        Self { min, max }
    }

    /// 複数の点からAABBを作成
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

        for p in points.iter().skip(1) {
            min_x = min_x.min(p.x());
            max_x = max_x.max(p.x());
            min_y = min_y.min(p.y());
            max_y = max_y.max(p.y());
            min_z = min_z.min(p.z());
            max_z = max_z.max(p.z());
        }

        Some(Self {
            min: Point3D::new(min_x, min_y, min_z),
            max: Point3D::new(max_x, max_y, max_z),
        })
    }

    /// 点がAABB内に含まれるか
    pub fn contains_point(&self, p: Point3D<T>) -> bool {
        (self.min.x() <= p.x() && p.x() <= self.max.x())
            && (self.min.y() <= p.y() && p.y() <= self.max.y())
            && (self.min.z() <= p.z() && p.z() <= self.max.z())
    }

    /// 点から AABB の境界（辺・面）までの最短距離
    ///
    /// 内部の点は最も近い辺・面までの距離、外部の点は AABB までの距離を返す。
    pub fn distance_to_boundary(&self, point: Point3D<T>) -> T {
        let coordinates = [point.x(), point.y(), point.z()];
        let mins = [self.min.x(), self.min.y(), self.min.z()];
        let maxs = [self.max.x(), self.max.y(), self.max.z()];

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
    pub fn classify_point(&self, point: Point3D<T>, tolerance: T) -> PointClassification {
        if self.distance_to_boundary(point) <= tolerance {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }

    /// 最小点を取得
    pub fn min(&self) -> Point3D<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max(&self) -> Point3D<T> {
        self.max
    }

    /// 幅（X軸方向のサイズ）
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 高さ（Y軸方向のサイズ）
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 深さ（Z軸方向のサイズ）
    pub fn depth(&self) -> T {
        self.max.z() - self.min.z()
    }

    /// 体積
    pub fn volume(&self) -> T {
        self.width() * self.height() * self.depth()
    }

    /// 中心点
    pub fn center(&self) -> Point3D<T> {
        let two = T::ONE + T::ONE;
        Point3D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
            (self.min.z() + self.max.z()) / two,
        )
    }

    /// 空のAABBか判定
    pub fn is_empty(&self) -> bool {
        self.min.x() > self.max.x() || self.min.y() > self.max.y() || self.min.z() > self.max.z()
    }

    /// 他のAABBを完全に含むか
    pub fn contains_aabb(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.max.x() >= other.max.x()
            && self.min.y() <= other.min.y()
            && self.max.y() >= other.max.y()
            && self.min.z() <= other.min.z()
            && self.max.z() >= other.max.z()
    }

    /// 他のAABBと交差するか
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
            && self.min.z() <= other.max.z()
            && self.max.z() >= other.min.z()
    }
}
