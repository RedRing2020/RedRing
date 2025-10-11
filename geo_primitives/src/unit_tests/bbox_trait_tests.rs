//! BBoxトレイトとBBoxOpsトレイトのテスト
//! 汎用バウンディングボックス操作のテスト

use crate::geometry3d::Point3D;
use crate::traits::BBox;

// テスト用のモック実装（テスト専用）
#[derive(Debug, Clone, PartialEq)]
struct MockBBox {
    min: Point3D<f64>,
    max: Point3D<f64>,
}

impl BBox<f64> for MockBBox {
    type Point = Point3D<f64>;

    fn min(&self) -> Self::Point {
        self.min
    }

    fn max(&self) -> Self::Point {
        self.max
    }

    fn new(min: Self::Point, max: Self::Point) -> Self {
        Self { min, max }
    }

    fn center(&self) -> Self::Point {
        Point3D::new(
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
            (self.min.z() + self.max.z()) / 2.0,
        )
    }

    fn volume(&self) -> f64 {
        (self.max.x() - self.min.x())
            * (self.max.y() - self.min.y())
            * (self.max.z() - self.min.z())
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y() && self.min.z() <= self.max.z()
    }
}

#[test]
fn test_basic_bbox_operations() {
    let bbox = MockBBox::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 3.0, 4.0));

    assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.max(), Point3D::new(2.0, 3.0, 4.0));
    assert_eq!(bbox.volume(), 24.0);
    assert_eq!(bbox.center(), Point3D::new(1.0, 1.5, 2.0));
    assert!(bbox.is_valid());
}

#[test]
fn test_invalid_bbox() {
    let invalid_bbox = MockBBox::new(Point3D::new(2.0, 3.0, 4.0), Point3D::new(0.0, 0.0, 0.0));

    assert!(!invalid_bbox.is_valid());
}
