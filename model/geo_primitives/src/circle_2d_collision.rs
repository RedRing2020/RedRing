//! Circle2D 衝突検出・距離計算 Foundation 実装
//!
//! BasicCollision トレイトの実装
//! Circle2D と他の幾何形状との組み合わせを実装

use crate::{Circle2D, LineSegment2D, Point2D};
use geo_foundation::{
    extensions::BasicCollision, Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Circle vs Point
impl<T: Scalar> BasicCollision<T, Point2D<T>> for Circle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 点が円周上にあるか判定
        let dx = point.x() - self.center().0;
        let dy = point.y() - self.center().1;
        let distance = (dx * dx + dy * dy).sqrt();
        (distance - self.radius()).abs() <= tolerance
    }

    fn overlaps(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 点が円内部または円周上にあるか判定
        let dx = point.x() - self.center().0;
        let dy = point.y() - self.center().1;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= self.radius() + tolerance
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        let dx = point.x() - self.center().0;
        let dy = point.y() - self.center().1;
        let distance = (dx * dx + dy * dy).sqrt();
        (distance - self.radius()).abs()
    }
}

// Circle vs Circle
impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Circle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Circle2D<T>, tolerance: T) -> bool {
        // 中心間距離を計算
        let dx = other.center().0 - self.center().0;
        let dy = other.center().1 - self.center().1;
        let center_distance = (dx * dx + dy * dy).sqrt();

        let radii_sum = self.radius() + other.radius();
        let radii_diff = (self.radius() - other.radius()).abs();

        // 2つの円が交差する条件
        center_distance <= radii_sum + tolerance && center_distance >= radii_diff - tolerance
    }

    fn overlaps(&self, other: &Circle2D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &Circle2D<T>) -> T {
        // 中心間距離から半径を差し引いた値
        let dx = other.center().0 - self.center().0;
        let dy = other.center().1 - self.center().1;
        let center_distance = (dx * dx + dy * dy).sqrt();
        let min_distance = center_distance - self.radius() - other.radius();

        min_distance.max(T::ZERO)
    }
}

// Circle vs LineSegment
impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for Circle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        self.distance_to(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        // 線分のいずれかの端点が円内部にあるか、線分が円と交差する
        let start = segment.start();
        let end = segment.end();
        let center = self.center();

        let start_inside = {
            let dx = start.0 - center.0;
            let dy = start.1 - center.1;
            (dx * dx + dy * dy).sqrt() <= self.radius() + tolerance
        };

        let end_inside = {
            let dx = end.0 - center.0;
            let dy = end.1 - center.1;
            (dx * dx + dy * dy).sqrt() <= self.radius() + tolerance
        };

        start_inside || end_inside || self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment2D<T>) -> T {
        // 円の中心から線分への最短距離を計算
        let center = self.center();
        let start = segment.start();
        let end = segment.end();

        // 線分のベクトル
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length_sq = dx * dx + dy * dy;

        if length_sq == T::ZERO {
            // 線分が点の場合
            let dist_x = center.0 - start.0;
            let dist_y = center.1 - start.1;
            let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();
            return (distance - self.radius()).max(T::ZERO);
        }

        // 線分上の最近点のパラメータを計算（0-1の範囲にクランプ）
        let t = {
            let dot = (center.0 - start.0) * dx + (center.1 - start.1) * dy;
            (dot / length_sq).max(T::ZERO).min(T::ONE)
        };

        // 線分上の最近点
        let closest_x = start.0 + t * dx;
        let closest_y = start.1 + t * dy;

        // 円の中心から最近点までの距離
        let dist_x = center.0 - closest_x;
        let dist_y = center.1 - closest_y;
        let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();

        // 円の表面までの距離
        (distance - self.radius()).max(T::ZERO)
    }
}
