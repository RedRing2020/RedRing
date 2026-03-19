//! Arc2D 衝突検出・距離計算 Foundation 実装
//!
//! BasicCollision トレイトの実装
//! Arc2D と他の幾何形状との組み合わせを実装

use crate::{Arc2D, Circle2D, Point2D};
use geo_contracts::BasicCollision;
use geo_contracts::{Arc2DProperties, Circle2DProperties, Scalar};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Arc vs Point
impl<T: Scalar> BasicCollision<T, Point2D<T>> for Arc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 簡易実装：点が基底円の円周上にあるかチェック
        let (center_x, center_y) = <Self as Arc2DProperties<T>>::center(self);
        let dx = point.x() - center_x;
        let dy = point.y() - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        (distance - self.radius_internal()).abs() <= tolerance
    }

    fn overlaps(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        // 簡易実装：基底円への距離
        let (center_x, center_y) = <Self as Arc2DProperties<T>>::center(self);
        let dx = point.x() - center_x;
        let dy = point.y() - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        (distance - self.radius_internal()).abs()
    }
}

// Arc vs Circle
impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Arc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        // 簡易実装：基底円と円の衝突判定
        let (center1_x, center1_y) = <Self as Arc2DProperties<T>>::center(self);
        let (center2_x, center2_y) = circle.center();

        let dx = center2_x - center1_x;
        let dy = center2_y - center1_y;
        let center_distance = (dx * dx + dy * dy).sqrt();

        let radii_sum = self.radius_internal() + circle.radius();
        let radii_diff = (self.radius_internal() - circle.radius()).abs();

        center_distance <= radii_sum + tolerance && center_distance >= radii_diff - tolerance
    }

    fn overlaps(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        // 簡易実装：中心間距離から半径を引く
        let (center1_x, center1_y) = <Self as Arc2DProperties<T>>::center(self);
        let (center2_x, center2_y) = circle.center();

        let dx = center2_x - center1_x;
        let dy = center2_y - center1_y;
        let center_distance = (dx * dx + dy * dy).sqrt();

        let min_distance = center_distance - self.radius() - circle.radius();
        min_distance.max(T::ZERO)
    }
}
