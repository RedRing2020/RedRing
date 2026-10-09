//! 2次元軸平行境界ボックス（Aabb2D）
//!
//! Foundation Pattern に基づく Aabb2D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

use crate::Point2D;
use analysis::abstract_types::Scalar;
use geo_contracts::{
    default_distance_tolerance, Aabb2DDerived, Aabb2DProperties, Aabb2DRelation, Contains,
    PointClassification,
};

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

    /// 点から AABB の境界（辺・面）までの最短距離
    ///
    /// 内部の点は最も近い辺・面までの距離、外部の点は AABB までの距離を返す。
    pub fn distance_to_boundary(&self, point: &Point2D<T>) -> T {
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
    pub fn classify_point(&self, point: &Point2D<T>, tolerance: T) -> PointClassification {
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

impl<T: Scalar> Aabb2DProperties<T> for Aabb2D<T> {
    type Point2D = Point2D<T>;

    fn min(&self) -> Self::Point2D {
        self.min
    }

    fn max(&self) -> Self::Point2D {
        self.max
    }
}

impl<T: Scalar> Aabb2DDerived<T> for Aabb2D<T> {
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

    fn is_valid(&self) -> bool {
        !(self.min.x() > self.max.x() || self.min.y() > self.max.y())
    }
}

impl<T: Scalar> Aabb2DRelation<T> for Aabb2D<T> {
    type Point2D = Point2D<T>;

    fn contains_point(&self, point: &Self::Point2D) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    fn classify_point(&self, point: &Self::Point2D) -> PointClassification {
        Aabb2D::classify_point(self, point, default_distance_tolerance::<T>())
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.max.x() >= other.max.x()
            && self.min.y() <= other.min.y()
            && self.max.y() >= other.max.y()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
    }
}

impl<T: Scalar> Contains<Point2D<T>> for Aabb2D<T> {
    fn contains(&self, target: &Point2D<T>) -> bool {
        self.contains_point(target)
    }
}

impl<T: Scalar> Contains<Aabb2D<T>> for Aabb2D<T> {
    fn contains(&self, target: &Aabb2D<T>) -> bool {
        self.contains_aabb(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{Aabb2DDerived, Aabb2DProperties, Aabb2DRelation, Contains};

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

    #[test]
    fn test_aabb2d_capability_traits() {
        let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 6.0));
        let inner = Aabb2D::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 5.0));

        assert_eq!(Aabb2DProperties::min(&outer), Point2D::new(0.0, 0.0));
        assert_eq!(Aabb2DProperties::max(&outer), Point2D::new(4.0, 6.0));
        assert_eq!(Aabb2DDerived::width(&outer), 4.0);
        assert_eq!(Aabb2DDerived::height(&outer), 6.0);
        assert_eq!(Aabb2DDerived::area(&outer), 24.0);
        assert_eq!(Aabb2DDerived::center(&outer), Point2D::new(2.0, 3.0));
        assert!(Aabb2DDerived::is_valid(&outer));
        assert!(Aabb2DRelation::contains_point(
            &outer,
            &Point2D::new(2.0, 3.0)
        ));
        assert!(Aabb2DRelation::contains_bbox(&outer, &inner));
        assert!(Aabb2DRelation::intersects(&outer, &inner));
    }

    #[test]
    fn test_contains_trait_for_point_and_aabb() {
        let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 4.0));
        let inner = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));

        assert!(Contains::contains(&outer, &Point2D::new(2.0, 2.0)));
        assert!(Contains::contains(&outer, &inner));
        assert!(outer.contains(&Point2D::new(2.0, 2.0)));
        assert!(outer.contains(&inner));
    }

    #[test]
    fn classify_point_uses_distance_to_boundary() {
        use geo_contracts::PointClassification;

        let aabb = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 2.0));
        let tolerance = 1e-3;
        let classify = |x: f64, y: f64| aabb.classify_point(&Point2D::new(x, y), tolerance);

        assert_eq!(classify(2.0, 1.0), PointClassification::Inside);
        assert_eq!(classify(4.0, 1.0), PointClassification::OnBoundary);
        assert_eq!(classify(5.0, 1.0), PointClassification::Outside);

        // 辺・角までの距離が許容誤差以内なら、内側・外側のどちらからでも OnBoundary
        assert_eq!(
            classify(4.0 - 0.5 * tolerance, 1.0),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(4.0 + 0.5 * tolerance, 1.0),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(4.0 - 2.0 * tolerance, 1.0),
            PointClassification::Inside
        );
        assert_eq!(
            classify(4.0 + 2.0 * tolerance, 1.0),
            PointClassification::Outside
        );
        // 角の近くは角までの距離で判定する（各辺から 0.6 / 0.8 倍ずつ離れた点）
        assert_eq!(
            classify(4.0 + 0.6 * tolerance, 2.0 + 0.6 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(4.0 + 0.8 * tolerance, 2.0 + 0.8 * tolerance),
            PointClassification::Outside
        );

        // 境界までの距離
        assert!((aabb.distance_to_boundary(&Point2D::new(1.0, 0.5)) - 0.5).abs() < 1e-12);
        assert!((aabb.distance_to_boundary(&Point2D::new(7.0, 6.0)) - 5.0).abs() < 1e-12);

        // contains_point は許容誤差 0 の分類が Outside でないことと一致する
        for (x, y) in [(2.0, 1.0), (4.0, 1.0), (4.0, 2.0), (5.0, 1.0)] {
            let point = Point2D::new(x, y);
            assert_eq!(
                aabb.contains_point(&point),
                aabb.classify_point(&point, 0.0) != PointClassification::Outside
            );
        }

        // trait定義は既定の距離トレランスで判定する
        assert_eq!(
            Aabb2DRelation::classify_point(&aabb, &Point2D::new(0.0, 1.0)),
            PointClassification::OnBoundary
        );
    }
}
