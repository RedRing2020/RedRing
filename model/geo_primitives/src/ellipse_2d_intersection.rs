//! Ellipse2D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{Arc2D, Circle2D, Ellipse2D, LineSegment2D, Point2D, Triangle2D};
use geo_foundation::{
    core::arc_traits::Arc2DProperties,
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// Ellipse2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        if self.distance_to_point(point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Ellipse2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, circle: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        let center = Point2D::new(circle.center().0, circle.center().1);

        if self.distance_to_point(&center) <= circle.radius() + tolerance {
            Some(center) // 簡易実装: 円の中心を返す
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let center = Point2D::new(circle.center().0, circle.center().1);

        if self.distance_to_point(&center) <= circle.radius() + tolerance {
            vec![center] // 簡易実装: 円の中心のみ
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, arc: &Arc2D<T>, tolerance: T) -> Option<Self::Point> {
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        if self.distance_to_point(&center) <= arc.radius() + tolerance {
            Some(center) // 簡易実装: 円弧の中心を返す
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, arc: &Arc2D<T>, _tolerance: T) -> Vec<Self::Point> {
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        if self.contains_point(&center, T::EPSILON) {
            vec![center] // 簡易実装
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        // 端点が楕円上にあるか確認
        if self.distance_to_point(&start) <= tolerance {
            return Some(start);
        }
        if self.distance_to_point(&end) <= tolerance {
            return Some(end);
        }

        // 簡易実装: 中点を返す
        let mid = Point2D::new(
            (start.x() + end.x()) / (T::ONE + T::ONE),
            (start.y() + end.y()) / (T::ONE + T::ONE),
        );

        if self.distance_to_point(&mid) <= tolerance {
            Some(mid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        if self.distance_to_point(&start) <= tolerance {
            intersections.push(start);
        }
        if self.distance_to_point(&end) <= tolerance {
            intersections.push(end);
        }

        intersections
    }
}

// ============================================================================
// Ellipse2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 楕円の中心を返す
        if triangle.contains_point(&self.center()) {
            Some(self.center())
        } else {
            let centroid = triangle.centroid();
            if self.distance_to_point(&centroid) <= tolerance {
                Some(centroid)
            } else {
                None
            }
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // 三角形の頂点が楕円上にあるか確認
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        if self.distance_to_point(&va) <= tolerance {
            intersections.push(va);
        }
        if self.distance_to_point(&vb) <= tolerance {
            intersections.push(vb);
        }
        if self.distance_to_point(&vc) <= tolerance {
            intersections.push(vc);
        }

        intersections
    }
}

// ============================================================================
// Ellipse2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Ellipse2D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 中心間の中点を返す
        let mid = Point2D::new(
            (self.center().x() + other.center().x()) / (T::ONE + T::ONE),
            (self.center().y() + other.center().y()) / (T::ONE + T::ONE),
        );

        if self.distance_to_point(&mid) <= tolerance {
            Some(mid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse2D<T>> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, _other: &Ellipse2D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: 空のベクタを返す（楕円同士の交点計算は複雑）
        Vec::new()
    }
}

// ============================================================================
// SelfIntersection
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Ellipse2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 楕円は自己交差しない
    }
}
