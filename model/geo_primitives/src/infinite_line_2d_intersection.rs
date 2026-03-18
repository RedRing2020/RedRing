//! InfiniteLine2D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{
    Arc2D, Circle2D, Ellipse2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D,
};
use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};
use geo_foundation::{
    core::arc_traits::Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// InfiniteLine2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, point: &Point2D<T>, tolerance: T) -> Option<Self::Point> {
        if self.contains_point(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// InfiniteLine2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for InfiniteLine2D<T> {
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

impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let projected = self.project_point(&center);

        let dist_to_center = self.distance_to_point(&center);

        if dist_to_center > circle.radius() + tolerance {
            return Vec::new();
        }

        if (dist_to_center - circle.radius()).abs() <= tolerance {
            // 接する場合
            return vec![projected];
        }

        // 2点で交わる場合
        let half_chord =
            (circle.radius() * circle.radius() - dist_to_center * dist_to_center).sqrt();
        let dir = self.direction_internal();

        let p1 = Point2D::new(
            projected.x() + half_chord * dir.x(),
            projected.y() + half_chord * dir.y(),
        );
        let p2 = Point2D::new(
            projected.x() - half_chord * dir.x(),
            projected.y() - half_chord * dir.y(),
        );

        vec![p1, p2]
    }
}

// ============================================================================
// InfiniteLine2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for InfiniteLine2D<T> {
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

impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for InfiniteLine2D<T> {
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
// InfiniteLine2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        let s1 = Point2D::new(segment.start().0, segment.start().1);
        let s2 = Point2D::new(segment.end().0, segment.end().1);

        let line_point = self.point_internal();
        let line_dir = self.direction_internal();

        let d1 = Vector2D::new(line_dir.x(), line_dir.y());
        let d2 = Vector2D::from_points(s1, s2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t2 = ((s1.x() - line_point.x()) * d1y - (s1.y() - line_point.y()) * d1x) / denominator;

        if t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance {
            Some(Point2D::new(s1.x() + t2 * d2x, s1.y() + t2 * d2y))
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Vec<Self::Point> {
        if let Some(point) = self.intersection_with(segment, tolerance) {
            vec![point]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// InfiniteLine2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Option<Self::Point> {
        let centroid = triangle.centroid();

        if self.contains_point(&centroid, tolerance) {
            Some(centroid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // 三角形の頂点が直線上にあるか確認
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        if self.contains_point(&va, tolerance) {
            intersections.push(va);
        }
        if self.contains_point(&vb, tolerance) {
            intersections.push(vb);
        }
        if self.contains_point(&vc, tolerance) {
            intersections.push(vc);
        }

        intersections
    }
}

// ============================================================================
// InfiniteLine2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> Option<Self::Point> {
        let center = ellipse.center_internal();

        if self.distance_to_point(&center) <= ellipse.semi_major_internal() + tolerance {
            Some(center) // 簡易実装: 楕円の中心を返す
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, ellipse: &Ellipse2D<T>, _tolerance: T) -> Vec<Self::Point> {
        let center = ellipse.center_internal();

        if self.contains_point(&center, T::EPSILON) {
            vec![center] // 簡易実装
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// InfiniteLine2D vs Ray2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, ray: &Ray2D<T>, tolerance: T) -> Option<Self::Point> {
        let origin = ray.origin_internal();
        let ray_dir = ray.direction_internal();

        let line_point = self.point_internal();
        let line_dir = self.direction_internal();

        let d1 = Vector2D::new(line_dir.x(), line_dir.y());
        let d2 = Vector2D::new(ray_dir.x(), ray_dir.y());

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t2 = ((origin.x() - line_point.x()) * d1y - (origin.y() - line_point.y()) * d1x)
            / denominator;

        if t2 >= T::ZERO - tolerance {
            Some(Point2D::new(origin.x() + t2 * d2x, origin.y() + t2 * d2y))
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ray2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, ray: &Ray2D<T>, tolerance: T) -> Vec<Self::Point> {
        if let Some(point) = self.intersection_with(ray, tolerance) {
            vec![point]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// InfiniteLine2D vs InfiniteLine2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, InfiniteLine2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &InfiniteLine2D<T>, _tolerance: T) -> Option<Self::Point> {
        let p1 = self.point_internal();
        let p2 = other.point_internal();
        let dir1 = self.direction_internal();
        let dir2 = other.direction_internal();

        let d1 = Vector2D::new(dir1.x(), dir1.y());
        let d2 = Vector2D::new(dir2.x(), dir2.y());

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t1 = ((p2.x() - p1.x()) * d2y - (p2.y() - p1.y()) * d2x) / denominator;

        Some(Point2D::new(p1.x() + t1 * d1x, p1.y() + t1 * d1y))
    }
}

impl<T: Scalar> MultipleIntersection<T, InfiniteLine2D<T>> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &InfiniteLine2D<T>, tolerance: T) -> Vec<Self::Point> {
        if let Some(point) = self.intersection_with(other, tolerance) {
            vec![point]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// SelfIntersection
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for InfiniteLine2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 無限直線は自己交差しない
    }
}
