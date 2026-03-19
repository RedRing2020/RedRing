//! EllipseArc2D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! 委譲パターンで Ellipse2D の実装を再利用し、角度範囲でフィルタリング

use crate::{Arc2D, Circle2D, Ellipse2D, EllipseArc2D, LineSegment2D, Point2D, Triangle2D};
use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};
use geo_contracts::Scalar;

// ============================================================================
// EllipseArc2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        // 1. 基底楕円との判定に委譲
        if let Some(p) = self.ellipse().intersection_with(point, tolerance) {
            // 2. 角度範囲内かチェック
            if self.point_in_angle_range(&p, tolerance) {
                return Some(p);
            }
        }
        None
    }
}

// ============================================================================
// EllipseArc2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, circle: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections = <Self as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
            self, circle, tolerance,
        );
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円との交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, Circle2D<T>>>::intersections_with(
                self.ellipse(),
                circle,
                tolerance,
            );

        // 2. 角度範囲内の交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, arc: &Arc2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections =
            <Self as MultipleIntersection<T, Arc2D<T>>>::intersections_with(self, arc, tolerance);
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, arc: &Arc2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円との交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, Arc2D<T>>>::intersections_with(
                self.ellipse(),
                arc,
                tolerance,
            );

        // 2. 角度範囲内の交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections = <Self as MultipleIntersection<T, Ellipse2D<T>>>::intersections_with(
            self, ellipse, tolerance,
        );
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円との交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, Ellipse2D<T>>>::intersections_with(
                self.ellipse(),
                ellipse,
                tolerance,
            );

        // 2. 角度範囲内の交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc2D vs EllipseArc2D (自己との交差)
// ============================================================================

impl<T: Scalar> BasicIntersection<T, EllipseArc2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &EllipseArc2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections = <Self as MultipleIntersection<T, EllipseArc2D<T>>>::intersections_with(
            self, other, tolerance,
        );
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, EllipseArc2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &EllipseArc2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円同士の交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, Ellipse2D<T>>>::intersections_with(
                self.ellipse(),
                other.ellipse(),
                tolerance,
            );

        // 2. 両方の楕円弧の角度範囲内にある交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| {
                self.point_in_angle_range(p, tolerance) && other.point_in_angle_range(p, tolerance)
            })
            .collect()
    }
}

// ============================================================================
// EllipseArc2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections = <Self as MultipleIntersection<T, LineSegment2D<T>>>::intersections_with(
            self, segment, tolerance,
        );
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円との交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, LineSegment2D<T>>>::intersections_with(
                self.ellipse(),
                segment,
                tolerance,
            );

        // 2. 角度範囲内の交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 複数交点から最初の1つを取得
        let intersections = <Self as MultipleIntersection<T, Triangle2D<T>>>::intersections_with(
            self, triangle, tolerance,
        );
        intersections.into_iter().next()
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle2D<T>> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Vec<Self::Point> {
        // 1. 基底楕円との交点を取得
        let ellipse_intersections =
            <Ellipse2D<T> as MultipleIntersection<T, Triangle2D<T>>>::intersections_with(
                self.ellipse(),
                triangle,
                tolerance,
            );

        // 2. 角度範囲内の交点のみフィルタリング
        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for EllipseArc2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 楕円弧は自己交差しない
        Vec::new()
    }
}
