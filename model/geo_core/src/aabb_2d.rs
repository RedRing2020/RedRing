//! 2次元軸平行境界ボックス（Aabb2D）
//!
//! Foundation Pattern に基づく Aabb2D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

use crate::Point2D;
use analysis::abstract_types::Scalar;
use geo_foundation::commons::Aabb2DTrait;

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

    /// 点からAABBを作成
    pub fn from_point(point: Point2D<T>) -> Self {
        Self::new(point, point)
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
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
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

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> Aabb2DTrait<T> for Aabb2D<T> {
    type Point2D = Point2D<T>;

    fn min(&self) -> Self::Point2D {
        self.min
    }

    fn max(&self) -> Self::Point2D {
        self.max
    }

    fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    fn area(&self) -> T {
        (self.max.x() - self.min.x()) * (self.max.y() - self.min.y())
    }

    fn center(&self) -> Self::Point2D {
        let two = T::ONE + T::ONE;
        Point2D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    fn contains_point(&self, point: &Self::Point2D) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.max.x() >= other.max.x()
            && self.min.y() <= other.min.y()
            && self.max.y() >= other.max.y()
    }

    fn is_valid(&self) -> bool {
        !(self.min.x() > self.max.x() || self.min.y() > self.max.y())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb2d_creation() {
        let min = Point2D::new(0.0, 0.0);
        let max = Point2D::new(1.0, 2.0);
        let aabb = Aabb2D::new(min, max);
        assert_eq!(aabb.min_point(), min);
        assert_eq!(aabb.max_point(), max);
    }

    #[test]
    fn test_aabb2d_from_point() {
        let point = Point2D::new(1.0, 2.0);
        let aabb = Aabb2D::from_point(point);
        assert_eq!(aabb.min_point(), point);
        assert_eq!(aabb.max_point(), point);
        assert_eq!(aabb.width(), 0.0);
    }

    #[test]
    fn test_aabb2d_dimensions() {
        let min = Point2D::new(1.0, 2.0);
        let max = Point2D::new(4.0, 7.0);
        let aabb = Aabb2D::new(min, max);
        assert_eq!(aabb.width(), 3.0);
        assert_eq!(aabb.height(), 5.0);
        assert_eq!(aabb.area(), 15.0);
    }

    #[test]
    fn test_aabb2d_center() {
        let min = Point2D::new(0.0, 0.0);
        let max = Point2D::new(2.0, 4.0);
        let aabb = Aabb2D::new(min, max);
        let center = aabb.center();
        assert_eq!(center.x(), 1.0);
        assert_eq!(center.y(), 2.0);
    }

    #[test]
    fn test_contains_point() {
        let min = Point2D::new(0.0, 0.0);
        let max = Point2D::new(2.0, 2.0);
        let aabb = Aabb2D::new(min, max);
        assert!(aabb.contains_point(&Point2D::new(1.0, 1.0)));
        assert!(aabb.contains_point(&Point2D::new(0.0, 0.0)));
        assert!(aabb.contains_point(&Point2D::new(2.0, 2.0)));
        assert!(!aabb.contains_point(&Point2D::new(3.0, 1.0)));
    }

    #[test]
    fn test_intersects() {
        let aabb1 = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0));
        let aabb2 = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
        let aabb3 = Aabb2D::new(Point2D::new(3.0, 3.0), Point2D::new(4.0, 4.0));
        assert!(aabb1.intersects(&aabb2));
        assert!(aabb2.intersects(&aabb1));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_contains_aabb() {
        let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 4.0));
        let inner = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
        let partial = Aabb2D::new(Point2D::new(2.0, 2.0), Point2D::new(5.0, 5.0));
        assert!(outer.contains_aabb(&inner));
        assert!(!outer.contains_aabb(&partial));
        assert!(!inner.contains_aabb(&outer));
    }

    #[test]
    fn test_is_empty() {
        let valid = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
        let invalid = Aabb2D::new(Point2D::new(1.0, 0.0), Point2D::new(0.0, 1.0));
        assert!(!valid.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_from_points() {
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(4.0, 5.0),
            Point2D::new(0.0, 1.0),
        ];
        let aabb = Aabb2D::from_points(&points).unwrap();
        assert_eq!(aabb.min_point(), Point2D::new(0.0, 1.0));
        assert_eq!(aabb.max_point(), Point2D::new(4.0, 5.0));
    }

    #[test]
    fn test_from_empty_points() {
        let points: Vec<Point2D<f64>> = vec![];
        assert!(Aabb2D::from_points(&points).is_none());
    }
}
