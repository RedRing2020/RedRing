//! Arc交点計算統一Foundation実装
//!
//! 統一Intersection Foundation システムによる交点計算
//! 全幾何プリミティブで共通利用可能な統一インターフェース

use crate::{Arc2D, Circle2D, Point2D, Vector2D};
use geo_foundation::{
    abstract_types::foundation::{BasicIntersection, MultipleIntersection, SelfIntersection},
    PointDistance, Scalar,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Arc vs Point（単一交点判定）
impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        if self.point_on_boundary(other, tolerance) {
            Some(*other)
        } else {
            None
        }
    }
}

// Arc vs Circle（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点計算の最初の要素を返す
        let intersections = <Self as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
            self, other, tolerance,
        );

        intersections.into_iter().next()
    }
}

// Arc vs Arc（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Arc2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点計算の最初の要素を返す
        let intersections =
            <Self as MultipleIntersection<T, Arc2D<T>>>::intersections_with(self, other, tolerance);

        intersections.into_iter().next()
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// Arc vs Circle（複数交点）
impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut result = Vec::new();

        // 1. 基底円同士の交点を計算
        let base_circle = self.circle();
        let circle_intersections = calculate_circle_circle_intersections(base_circle, other);

        // 2. 交点が円弧の角度範囲内にあるかを確認
        for intersection in circle_intersections {
            let center = base_circle.center();
            let to_intersection =
                Vector2D::new(intersection.x() - center.x(), intersection.y() - center.y());
            let angle = to_intersection.angle();

            if self.angle_in_range_with_tolerance(angle, tolerance) {
                result.push(intersection);
            }
        }

        result
    }
}

// Arc vs Arc（複数交点）
impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &Arc2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut result = Vec::new();

        // 1. 基底円同士の交点を計算
        let base_circle1 = self.circle();
        let base_circle2 = other.circle();
        let circle_intersections =
            calculate_circle_circle_intersections(base_circle1, base_circle2);

        // 2. 交点が両方の円弧の角度範囲内にあるかを確認
        for intersection in circle_intersections {
            // 最初の円弧での角度確認
            let center1 = base_circle1.center();
            let to_intersection1 = Vector2D::new(
                intersection.x() - center1.x(),
                intersection.y() - center1.y(),
            );
            let angle1 = to_intersection1.angle();

            // 2番目の円弧での角度確認
            let center2 = base_circle2.center();
            let to_intersection2 = Vector2D::new(
                intersection.x() - center2.x(),
                intersection.y() - center2.y(),
            );
            let angle2 = to_intersection2.angle();

            if self.angle_in_range_with_tolerance(angle1, tolerance)
                && other.angle_in_range_with_tolerance(angle2, tolerance)
            {
                result.push(intersection);
            }
        }

        result
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Arc2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 通常の円弧は自己交差しない
        // フル円弧（360度）の場合は開始点=終了点だが、これは通常自己交差とは考えない
        Vec::new()
    }
}

// ============================================================================
// 幾何計算ヘルパー関数
// ============================================================================

/// 円と円の交点を計算（最大2点）
fn calculate_circle_circle_intersections<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center1 = circle1.center();
    let center2 = circle2.center();
    let r1 = circle1.radius();
    let r2 = circle2.radius();

    // 中心間距離
    let dx = center2.x() - center1.x();
    let dy = center2.y() - center1.y();
    let d = (dx * dx + dy * dy).sqrt();

    // 交点判定
    if d > r1 + r2 || d < (r1 - r2).abs() || d == T::ZERO {
        // 交点なし
        return result;
    }

    // 交点計算
    let a = (r1 * r1 - r2 * r2 + d * d) / ((T::ONE + T::ONE) * d);
    let h_squared = r1 * r1 - a * a;

    if h_squared < T::ZERO {
        // 交点なし（数値誤差対応）
        return result;
    }

    let h = h_squared.sqrt();

    // 中点
    let px = center1.x() + a * dx / d;
    let py = center1.y() + a * dy / d;

    if h == T::ZERO {
        // 接点（1点）
        result.push(Point2D::new(px, py));
    } else {
        // 2交点
        let intersection1 = Point2D::new(px + h * dy / d, py - h * dx / d);
        let intersection2 = Point2D::new(px - h * dy / d, py + h * dx / d);

        result.push(intersection1);
        result.push(intersection2);
    }

    result
}

// angle_in_range methods are implemented in arc_2d_collision.rs
