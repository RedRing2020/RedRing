//! 3次元軸平行境界ボックス（Aabb3D）
//!
//! Foundation Pattern に基づく Aabb3D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

use analysis::abstract_types::Scalar;
use geo_contracts::{
    default_distance_tolerance, Aabb3DDerived, Aabb3DProperties, Aabb3DRelation, Contains,
    PointClassification,
};

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
    pub fn contains_point(&self, p: &Point3D<T>) -> bool {
        (self.min.x() <= p.x() && p.x() <= self.max.x())
            && (self.min.y() <= p.y() && p.y() <= self.max.y())
            && (self.min.z() <= p.z() && p.z() <= self.max.z())
    }

    /// 点から AABB の境界（辺・面）までの最短距離
    ///
    /// 内部の点は最も近い辺・面までの距離、外部の点は AABB までの距離を返す。
    pub fn distance_to_boundary(&self, point: &Point3D<T>) -> T {
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
    pub fn classify_point(&self, point: &Point3D<T>, tolerance: T) -> PointClassification {
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

impl<T: Scalar> Aabb3DProperties<T> for Aabb3D<T> {
    type Point3D = Point3D<T>;

    fn min(&self) -> Self::Point3D {
        self.min
    }

    fn max(&self) -> Self::Point3D {
        self.max
    }
}

impl<T: Scalar> Aabb3DDerived<T> for Aabb3D<T> {
    fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    fn depth(&self) -> T {
        self.max.z() - self.min.z()
    }

    fn volume(&self) -> T {
        (self.max.x() - self.min.x())
            * (self.max.y() - self.min.y())
            * (self.max.z() - self.min.z())
    }

    fn center(&self) -> Self::Point3D {
        let two = T::ONE + T::ONE;
        Point3D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
            (self.min.z() + self.max.z()) / two,
        )
    }

    fn is_valid(&self) -> bool {
        !(self.min.x() > self.max.x() || self.min.y() > self.max.y() || self.min.z() > self.max.z())
    }
}

impl<T: Scalar> Aabb3DRelation<T> for Aabb3D<T> {
    type Point3D = Point3D<T>;

    fn contains_point(&self, point: &Self::Point3D) -> bool {
        (self.min.x() <= point.x() && point.x() <= self.max.x())
            && (self.min.y() <= point.y() && point.y() <= self.max.y())
            && (self.min.z() <= point.z() && point.z() <= self.max.z())
    }

    fn classify_point(&self, point: &Self::Point3D) -> PointClassification {
        Aabb3D::classify_point(self, point, default_distance_tolerance::<T>())
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.max.x() >= other.max.x()
            && self.min.y() <= other.min.y()
            && self.max.y() >= other.max.y()
            && self.min.z() <= other.min.z()
            && self.max.z() >= other.max.z()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
            && self.min.z() <= other.max.z()
            && self.max.z() >= other.min.z()
    }
}

impl<T: Scalar> Contains<Point3D<T>> for Aabb3D<T> {
    fn contains(&self, target: &Point3D<T>) -> bool {
        self.contains_point(target)
    }
}

impl<T: Scalar> Contains<Aabb3D<T>> for Aabb3D<T> {
    fn contains(&self, target: &Aabb3D<T>) -> bool {
        self.contains_aabb(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{Aabb3DDerived, Aabb3DProperties, Aabb3DRelation, Contains};

    #[test]
    fn test_aabb3d_creation() {
        let min = Point3D::new(0.0, 0.0, 0.0);
        let max = Point3D::new(1.0, 2.0, 3.0);
        let aabb = Aabb3D::new(min, max);
        assert_eq!(aabb.min(), min);
        assert_eq!(aabb.max(), max);
    }

    #[test]
    fn test_aabb3d_dimensions() {
        let min = Point3D::new(1.0, 2.0, 3.0);
        let max = Point3D::new(4.0, 7.0, 9.0);
        let aabb = Aabb3D::new(min, max);
        assert_eq!(aabb.width(), 3.0);
        assert_eq!(aabb.height(), 5.0);
        assert_eq!(aabb.depth(), 6.0);
        assert_eq!(aabb.volume(), 90.0);
    }

    #[test]
    fn test_aabb3d_center() {
        let min = Point3D::new(0.0, 0.0, 0.0);
        let max = Point3D::new(2.0, 4.0, 6.0);
        let aabb = Aabb3D::new(min, max);
        let center = aabb.center();
        assert_eq!(center.x(), 1.0);
        assert_eq!(center.y(), 2.0);
        assert_eq!(center.z(), 3.0);
    }

    #[test]
    fn test_contains() {
        let min = Point3D::new(0.0, 0.0, 0.0);
        let max = Point3D::new(2.0, 2.0, 2.0);
        let aabb = Aabb3D::new(min, max);
        assert!(aabb.contains_point(&Point3D::new(1.0, 1.0, 1.0)));
        assert!(aabb.contains_point(&Point3D::new(0.0, 0.0, 0.0)));
        assert!(aabb.contains_point(&Point3D::new(2.0, 2.0, 2.0)));
        assert!(!aabb.contains_point(&Point3D::new(3.0, 1.0, 1.0)));
    }

    #[test]
    fn test_from_points() {
        let points = vec![
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(4.0, 5.0, 6.0),
            Point3D::new(0.0, 1.0, 2.0),
        ];
        let aabb = Aabb3D::from_points(&points).unwrap();
        assert_eq!(aabb.min(), Point3D::new(0.0, 1.0, 2.0));
        assert_eq!(aabb.max(), Point3D::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_is_empty() {
        let valid = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0));
        let invalid = Aabb3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 1.0));
        assert!(!valid.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_aabb3d_capability_traits() {
        let outer = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 6.0, 8.0));
        let inner = Aabb3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(3.0, 5.0, 7.0));

        assert_eq!(Aabb3DProperties::min(&outer), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(Aabb3DProperties::max(&outer), Point3D::new(4.0, 6.0, 8.0));
        assert_eq!(Aabb3DDerived::width(&outer), 4.0);
        assert_eq!(Aabb3DDerived::height(&outer), 6.0);
        assert_eq!(Aabb3DDerived::depth(&outer), 8.0);
        assert_eq!(Aabb3DDerived::volume(&outer), 192.0);
        assert_eq!(Aabb3DDerived::center(&outer), Point3D::new(2.0, 3.0, 4.0));
        assert!(Aabb3DDerived::is_valid(&outer));
        assert!(Aabb3DRelation::contains_point(
            &outer,
            &Point3D::new(2.0, 3.0, 4.0)
        ));
        assert!(Aabb3DRelation::contains_bbox(&outer, &inner));
        assert!(Aabb3DRelation::intersects(&outer, &inner));
    }

    #[test]
    fn test_contains_trait_for_point_and_aabb() {
        let outer = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 4.0, 4.0));
        let inner = Aabb3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 3.0, 3.0));

        assert!(Contains::contains(&outer, &Point3D::new(2.0, 2.0, 2.0)));
        assert!(Contains::contains(&outer, &inner));
        assert!(outer.contains(&Point3D::new(2.0, 2.0, 2.0)));
        assert!(outer.contains(&inner));
    }

    #[test]
    fn classify_point_uses_distance_to_boundary() {
        use geo_contracts::PointClassification;

        let aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 2.0, 2.0));
        let tolerance = 1e-3;
        let classify =
            |x: f64, y: f64, z: f64| aabb.classify_point(&Point3D::new(x, y, z), tolerance);

        assert_eq!(classify(2.0, 1.0, 1.0), PointClassification::Inside);
        assert_eq!(classify(2.0, 1.0, 2.0), PointClassification::OnBoundary);
        assert_eq!(classify(2.0, 1.0, 3.0), PointClassification::Outside);
        assert_eq!(
            classify(2.0, 1.0, 2.0 - 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(2.0, 1.0, 2.0 + 0.5 * tolerance),
            PointClassification::OnBoundary
        );
        assert_eq!(
            classify(2.0, 1.0, 2.0 - 2.0 * tolerance),
            PointClassification::Inside
        );

        // 境界までの距離
        assert!((aabb.distance_to_boundary(&Point3D::new(2.0, 1.0, 0.5)) - 0.5).abs() < 1e-12);
        assert!((aabb.distance_to_boundary(&Point3D::new(6.0, 2.0, 0.0)) - 2.0).abs() < 1e-12);

        // contains_point は許容誤差 0 の分類が Outside でないことと一致する
        for (x, y, z) in [(2.0, 1.0, 1.0), (4.0, 2.0, 2.0), (4.5, 1.0, 1.0)] {
            let point = Point3D::new(x, y, z);
            assert_eq!(
                aabb.contains_point(&point),
                aabb.classify_point(&point, 0.0) != PointClassification::Outside
            );
        }

        assert_eq!(
            Aabb3DRelation::classify_point(&aabb, &Point3D::new(1.0, 1.0, 1.0)),
            PointClassification::Inside
        );
    }
}
