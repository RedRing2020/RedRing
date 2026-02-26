//! Arc2D 交点計算 Foundation 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! Arc2D と他の幾何形状との組み合わせを実装

use crate::{Arc2D, Circle2D, Point2D};
use geo_foundation::{
    Arc2DProperties, BasicIntersection, Circle2DProperties, MultipleIntersection, Scalar,
    SelfIntersection,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Arc vs Point（単一交点判定）
impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        // 点が円弧上にあれば、その点を交点として返す
        let (center_x, center_y) = <Self as Arc2DProperties<T>>::center(self);
        let dx = point.x() - center_x;
        let dy = point.y() - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // 1. 円周上にあるか確認
        if (distance - self.radius_internal()).abs() > tolerance {
            return None;
        }

        // 2. 角度範囲内にあるか確認
        if !self.contains_point_angle(*point) {
            return None;
        }

        Some(*point)
    }
}

// Arc vs Circle（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, circle: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        let intersections = <Self as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
            self, circle, tolerance,
        );
        intersections.into_iter().next()
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// Arc vs Circle（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Arc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装：基底円と円の交点を計算
        calculate_arc_circle_intersections(self, circle)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Arc2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 円弧は自己交差しない
        Vec::new()
    }
}

// ============================================================================
// 幾何計算ヘルパー関数
// ============================================================================

/// 円弧と円の交点を計算（角度範囲考慮版）
fn calculate_arc_circle_intersections<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    // 基底円と円の交点を計算
    let (arc_center_x, arc_center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let base_circle = Circle2D::new(
        Point2D::new(arc_center_x, arc_center_y),
        arc.radius_internal(),
    );

    if let Some(base_circle) = base_circle {
        let circle_intersections = calculate_circle_circle_intersections(&base_circle, circle);

        // 角度範囲内の交点のみをフィルタリング
        for point in circle_intersections {
            if arc.contains_point_angle(point) {
                result.push(point);
            }
        }
    }

    result
}

/// 円と円の交点を計算（Circle2D から再利用）
fn calculate_circle_circle_intersections<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center1 = circle1.center();
    let center2 = circle2.center();
    let r1 = circle1.radius();
    let r2 = circle2.radius();

    let dx = center2.0 - center1.0;
    let dy = center2.1 - center1.1;
    let d = (dx * dx + dy * dy).sqrt();

    if d > r1 + r2 || d < (r1 - r2).abs() || d == T::ZERO {
        return result;
    }

    let a = (r1 * r1 - r2 * r2 + d * d) / ((T::ONE + T::ONE) * d);
    let h_squared = r1 * r1 - a * a;

    if h_squared < T::ZERO {
        return result;
    }

    let h = h_squared.sqrt();
    let px = center1.0 + a * dx / d;
    let py = center1.1 + a * dy / d;

    if h == T::ZERO {
        result.push(Point2D::new(px, py));
    } else {
        result.push(Point2D::new(px + h * dy / d, py - h * dx / d));
        result.push(Point2D::new(px - h * dy / d, py + h * dx / d));
    }

    result
}
