//! EllipseArc2D Collision 実装
//!
//! BasicCollision トレイトの実装
//! 委譲パターンで Ellipse2D の実装を再利用

use crate::{Arc2D, Circle2D, Ellipse2D, EllipseArc2D, LineSegment2D, Point2D, Triangle2D};
use geo_foundation::{
    core::arc_core_traits::Arc2DProperties, extensions::BasicCollision, Circle2DProperties,
    LineSegment2DProperties, Scalar,
};

// ============================================================================
// EllipseArc2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 1. 基底楕円との判定に委譲
        if !self.ellipse().intersects(point, tolerance) {
            return false;
        }
        // 2. 角度範囲内かチェック
        self.point_in_angle_range(point, tolerance)
    }

    fn overlaps(&self, _point: &Point2D<T>, _tolerance: T) -> bool {
        false // 点は重なりを持たない
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        // 既存の distance_to_point メソッドを使用
        self.distance_to_point(point)
    }
}

// ============================================================================
// EllipseArc2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        // 1. 基底楕円との判定に委譲
        if !self.ellipse().intersects(circle, tolerance) {
            return false;
        }

        // 2. 円の中心が楕円弧の角度範囲に近い場合は交差の可能性あり
        // 簡易実装: 楕円との交差があれば、楕円弧とも交差する可能性
        // より厳密には、交点が角度範囲内か確認が必要
        true
    }

    fn overlaps(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        // 簡易実装: 楕円の overlap に委譲
        self.ellipse().overlaps(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        // 円の中心点への距離から半径を引く
        let center = Point2D::new(circle.center().0, circle.center().1);
        let dist_to_center = self.distance_to_point(&center);
        (dist_to_center - circle.radius()).max(T::ZERO)
    }
}

// ============================================================================
// EllipseArc2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円と円弧の基底円の判定
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        let dist = self.distance_to_point(&center);
        dist <= arc.radius() + tolerance
    }

    fn overlaps(&self, _arc: &Arc2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, arc: &Arc2D<T>) -> T {
        // 円弧の中心点への距離から半径を引く
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);
        let dist_to_center = self.distance_to_point(&center);
        (dist_to_center - arc.radius()).max(T::ZERO)
    }
}

// ============================================================================
// EllipseArc2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> bool {
        // 基底楕円との判定に委譲
        self.ellipse().intersects(ellipse, tolerance)
    }

    fn overlaps(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円の overlap に委譲
        self.ellipse().overlaps(ellipse, tolerance)
    }

    fn distance_to(&self, ellipse: &Ellipse2D<T>) -> T {
        // 楕円の中心点への距離を計算
        let center = ellipse.center();
        self.distance_to_point(&center)
    }
}

// ============================================================================
// EllipseArc2D vs EllipseArc2D (自己との衝突)
// ============================================================================

impl<T: Scalar> BasicCollision<T, EllipseArc2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &EllipseArc2D<T>, tolerance: T) -> bool {
        // 1. 基底楕円同士の判定
        if !self.ellipse().intersects(other.ellipse(), tolerance) {
            return false;
        }

        // 2. 角度範囲が重なるか確認
        // 簡易実装: 基底楕円が交差すれば楕円弧も交差する可能性あり
        true
    }

    fn overlaps(&self, other: &EllipseArc2D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円の overlap に委譲
        self.ellipse().overlaps(other.ellipse(), tolerance)
    }

    fn distance_to(&self, other: &EllipseArc2D<T>) -> T {
        // 簡易実装: 相手の中心点への距離
        let other_center = other.center();
        self.distance_to_point(&other_center)
    }
}

// ============================================================================
// EllipseArc2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        // 基底楕円との判定に委譲
        self.ellipse().intersects(segment, tolerance)
    }

    fn overlaps(&self, _segment: &LineSegment2D<T>, _tolerance: T) -> bool {
        false // 線分は重なりを持たない
    }

    fn distance_to(&self, segment: &LineSegment2D<T>) -> T {
        // 線分の両端点への距離の最小値
        let (start_x, start_y) = <LineSegment2D<T> as LineSegment2DProperties<T>>::start(segment);
        let (end_x, end_y) = <LineSegment2D<T> as LineSegment2DProperties<T>>::end(segment);

        let start_point = Point2D::new(start_x, start_y);
        let end_point = Point2D::new(end_x, end_y);

        let dist_to_start = self.distance_to_point(&start_point);
        let dist_to_end = self.distance_to_point(&end_point);

        dist_to_start.min(dist_to_end)
    }
}

// ============================================================================
// EllipseArc2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle2D<T>> for EllipseArc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        // 基底楕円との判定に委譲
        self.ellipse().intersects(triangle, tolerance)
    }

    fn overlaps(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円の overlap に委譲
        self.ellipse().overlaps(triangle, tolerance)
    }

    fn distance_to(&self, triangle: &Triangle2D<T>) -> T {
        // 基底楕円への距離に委譲
        self.ellipse().distance_to(triangle)
    }
}
