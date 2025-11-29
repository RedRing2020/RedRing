//! Circle2D 交点計算 Foundation 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! Circle2D と他の幾何形状との組み合わせを実装

use crate::{Circle2D, LineSegment2D, Point2D};
use geo_foundation::{
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Circle vs Point（単一交点判定）
impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Circle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        // 点が円周上にあれば、その点を交点として返す
        let dx = point.x() - self.center().0;
        let dy = point.y() - self.center().1;
        let distance = (dx * dx + dy * dy).sqrt();

        if (distance - self.radius()).abs() <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// Circle vs Circle（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Circle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点計算の最初の要素を返す
        let intersections = <Self as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
            self, other, tolerance,
        );

        intersections.into_iter().next()
    }
}

// Circle vs LineSegment（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for Circle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点計算の最初の要素を返す
        let intersections = <Self as MultipleIntersection<T, LineSegment2D<T>>>::intersections_with(
            self, segment, tolerance,
        );

        intersections.into_iter().next()
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// Circle vs Circle（複数交点）
impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Circle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &Circle2D<T>, _tolerance: T) -> Vec<Self::Point> {
        calculate_circle_circle_intersections(self, other)
    }
}

// Circle vs LineSegment（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for Circle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, segment: &LineSegment2D<T>, _tolerance: T) -> Vec<Self::Point> {
        calculate_circle_line_segment_intersections(self, segment)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Circle2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 円は自己交差しない
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
    let dx = center2.0 - center1.0;
    let dy = center2.1 - center1.1;
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
    let px = center1.0 + a * dx / d;
    let py = center1.1 + a * dy / d;

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

/// 円と線分の交点を計算（最大2点）
fn calculate_circle_line_segment_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    segment: &LineSegment2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start();
    let end = segment.end();

    // 線分のベクトル
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;

    // 始点から中心へのベクトル
    let fx = start.0 - center.0;
    let fy = start.1 - center.1;

    // 2次方程式の係数: a*t^2 + b*t + c = 0
    let a = dx * dx + dy * dy;
    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

    if discriminant < T::ZERO {
        // 交点なし
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    // t1, t2 を計算（線分上のパラメータ）
    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    // t が [0, 1] の範囲内にある交点のみを追加
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
