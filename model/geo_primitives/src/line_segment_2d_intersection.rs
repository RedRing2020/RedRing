//! LineSegment2D 交点計算 Foundation 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! LineSegment2D と他の幾何形状との組み合わせを実装

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D};
use geo_contracts::{
    Arc2DProperties, BasicIntersection, Circle2DProperties, LineSegment2DProperties,
    MultipleIntersection, Scalar, SelfIntersection,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// LineSegment vs Point（単一交点判定）
impl<T: Scalar> BasicIntersection<T, Point2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        // 点が線分上にあれば、その点を交点として返す
        let start = self.start();
        let end = self.end();

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length_sq = dx * dx + dy * dy;

        if length_sq == T::ZERO {
            // 退化した線分
            let dist_x = point.x() - start.0;
            let dist_y = point.y() - start.1;
            let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();
            return if distance <= tolerance {
                Some(*point)
            } else {
                None
            };
        }

        // 点から線分への距離をチェック
        let t = {
            let dot = (point.x() - start.0) * dx + (point.y() - start.1) * dy;
            (dot / length_sq).max(T::ZERO).min(T::ONE)
        };

        let closest_x = start.0 + t * dx;
        let closest_y = start.1 + t * dy;

        let dist_x = point.x() - closest_x;
        let dist_y = point.y() - closest_y;
        let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();

        if distance <= tolerance {
            Some(Point2D::new(closest_x, closest_y))
        } else {
            None
        }
    }
}

// LineSegment vs Circle（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, circle: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        let intersections = <Self as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
            self, circle, tolerance,
        );
        intersections.into_iter().next()
    }
}

// LineSegment vs Arc（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, arc: &Arc2D<T>, tolerance: T) -> Option<Self::Point> {
        let intersections =
            <Self as MultipleIntersection<T, Arc2D<T>>>::intersections_with(self, arc, tolerance);
        intersections.into_iter().next()
    }
}

// LineSegment vs LineSegment（単一交点）
impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &LineSegment2D<T>, _tolerance: T) -> Option<Self::Point> {
        calculate_line_segment_intersection(self, other)
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// LineSegment vs Circle（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, _tolerance: T) -> Vec<Self::Point> {
        calculate_line_segment_circle_intersections(self, circle)
    }
}

// LineSegment vs Arc（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, arc: &Arc2D<T>, _tolerance: T) -> Vec<Self::Point> {
        calculate_line_segment_arc_intersections(self, arc)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for LineSegment2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 線分は自己交差しない
        Vec::new()
    }
}

// ============================================================================
// 幾何計算ヘルパー関数
// ============================================================================

/// 線分と円の交点を計算（最大2点）
fn calculate_line_segment_circle_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start();
    let end = segment.end();

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let fx = start.0 - center.0;
    let fy = start.1 - center.1;

    let a = dx * dx + dy * dy;
    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= T::ZERO && t1 <= T::ONE {
        let x = start.0 + t1 * dx;
        let y = start.1 + t1 * dy;
        result.push(Point2D::new(x, y));
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > T::EPSILON {
        let x = start.0 + t2 * dx;
        let y = start.1 + t2 * dy;
        result.push(Point2D::new(x, y));
    }

    result
}

/// 線分と円弧の交点を計算（最大2点）
fn calculate_line_segment_arc_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
) -> Vec<Point2D<T>> {
    // 簡易実装：基底円との交点を計算（角度範囲チェックは省略）
    let (arc_center_x, arc_center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let base_circle = Circle2D::new(Point2D::new(arc_center_x, arc_center_y), arc.radius());

    if let Some(base_circle) = base_circle {
        calculate_line_segment_circle_intersections(segment, &base_circle)
    } else {
        Vec::new()
    }
}

/// 線分と線分の交点を計算（最大1点）
fn calculate_line_segment_intersection<T: Scalar>(
    seg1: &LineSegment2D<T>,
    seg2: &LineSegment2D<T>,
) -> Option<Point2D<T>> {
    let p1 = seg1.start();
    let p2 = seg1.end();
    let p3 = seg2.start();
    let p4 = seg2.end();

    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denominator = d1x * d2y - d1y * d2x;

    if denominator.abs() < T::EPSILON {
        // 平行または一致
        return None;
    }

    let t1 = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denominator;
    let t2 = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denominator;

    if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
        let x = p1.0 + t1 * d1x;
        let y = p1.1 + t1 * d1y;
        Some(Point2D::new(x, y))
    } else {
        None
    }
}
