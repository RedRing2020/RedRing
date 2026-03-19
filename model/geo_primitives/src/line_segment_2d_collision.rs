//! LineSegment2D 衝突検出・距離計算 Foundation 実装
//!
//! BasicCollision トレイトの実装
//! LineSegment2D と他の幾何形状との組み合わせを実装

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D};
use geo_contracts::BasicCollision;
use geo_contracts::{Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// LineSegment vs Point
impl<T: Scalar> BasicCollision<T, Point2D<T>> for LineSegment2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        // 点から線分への最短距離
        let start = self.start();
        let end = self.end();

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length_sq = dx * dx + dy * dy;

        if length_sq == T::ZERO {
            // 線分が点の場合（退化）
            let dist_x = point.x() - start.0;
            let dist_y = point.y() - start.1;
            return (dist_x * dist_x + dist_y * dist_y).sqrt();
        }

        // 線分上の最近点のパラメータ t ∈ [0, 1]
        let t = {
            let dot = (point.x() - start.0) * dx + (point.y() - start.1) * dy;
            (dot / length_sq).max(T::ZERO).min(T::ONE)
        };

        // 線分上の最近点
        let closest_x = start.0 + t * dx;
        let closest_y = start.1 + t * dy;

        // 点から最近点までの距離
        let dist_x = point.x() - closest_x;
        let dist_y = point.y() - closest_y;
        (dist_x * dist_x + dist_y * dist_y).sqrt()
    }
}

// LineSegment vs Circle
impl<T: Scalar> BasicCollision<T, Circle2D<T>> for LineSegment2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        // 線分の端点が円内にあるか、または交差する
        let start = self.start();
        let end = self.end();
        let center = circle.center();

        let start_inside = {
            let dx = start.0 - center.0;
            let dy = start.1 - center.1;
            (dx * dx + dy * dy).sqrt() <= circle.radius() + tolerance
        };

        let end_inside = {
            let dx = end.0 - center.0;
            let dy = end.1 - center.1;
            (dx * dx + dy * dy).sqrt() <= circle.radius() + tolerance
        };

        start_inside || end_inside || self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        // 線分の中心から円への最短距離を計算し、半径を引く
        let center = circle.center();
        let center_point = Point2D::new(center.0, center.1);

        let distance_to_center = self.distance_to(&center_point);
        (distance_to_center - circle.radius()).max(T::ZERO)
    }
}

// LineSegment vs Arc
impl<T: Scalar> BasicCollision<T, Arc2D<T>> for LineSegment2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        self.distance_to(arc) <= tolerance
    }

    fn overlaps(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        self.intersects(arc, tolerance)
    }

    fn distance_to(&self, arc: &Arc2D<T>) -> T {
        // 簡易実装：基底円との距離
        let (arc_center_x, arc_center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center_point = Point2D::new(arc_center_x, arc_center_y);

        let distance_to_center = self.distance_to(&center_point);
        (distance_to_center - arc.radius()).max(T::ZERO)
    }
}

// LineSegment vs LineSegment
impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for LineSegment2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &LineSegment2D<T>, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    fn overlaps(&self, other: &LineSegment2D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &LineSegment2D<T>) -> T {
        // 線分と線分の最短距離（簡易実装）
        // 4つの端点の組み合わせを計算
        let p1_start = self.start();
        let p1_end = self.end();
        let p2_start = other.start();
        let p2_end = other.end();

        // 各端点から相手の線分への距離の最小値
        let d1 = self.distance_to(&Point2D::new(p2_start.0, p2_start.1));
        let d2 = self.distance_to(&Point2D::new(p2_end.0, p2_end.1));
        let d3 = other.distance_to(&Point2D::new(p1_start.0, p1_start.1));
        let d4 = other.distance_to(&Point2D::new(p1_end.0, p1_end.1));

        d1.min(d2).min(d3).min(d4)
    }
}
