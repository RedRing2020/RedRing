//! Ray2D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{Arc2D, Circle2D, Ellipse2D, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D};
use geo_foundation::{
    core::arc_traits::Arc2DProperties,
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// Ray2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Ray2D<T> {
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
// Ray2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Ray2D<T> {
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

impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let origin = self.origin_internal();
        let direction = self.direction_internal();

        // Ray と円の交点計算
        let oc = origin - center;
        let dir_vec = Vector2D::new(direction.x(), direction.y());

        let a = dir_vec.dot(&dir_vec);
        let b = (oc.dot(&dir_vec)) * (T::ONE + T::ONE);
        let c = oc.dot(&oc) - circle.radius() * circle.radius();

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();
        let two_a = (T::ONE + T::ONE) * a;

        let t1 = (-b - sqrt_disc) / two_a;
        let t2 = (-b + sqrt_disc) / two_a;

        let mut intersections = Vec::new();

        if t1 >= T::ZERO - tolerance {
            let point = Point2D::new(
                origin.x() + t1 * direction.x(),
                origin.y() + t1 * direction.y(),
            );
            intersections.push(point);
        }

        if t2 >= T::ZERO - tolerance && (t2 - t1).abs() > tolerance {
            let point = Point2D::new(
                origin.x() + t2 * direction.x(),
                origin.y() + t2 * direction.y(),
            );
            intersections.push(point);
        }

        intersections
    }
}

// ============================================================================
// Ray2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for Ray2D<T> {
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

impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for Ray2D<T> {
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
// Ray2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        let s1 = Point2D::new(segment.start().0, segment.start().1);
        let s2 = Point2D::new(segment.end().0, segment.end().1);

        let origin = self.origin_internal();
        let direction = self.direction_internal();

        let d1 = Vector2D::new(direction.x(), direction.y());
        let d2 = Vector2D::from_points(s1, s2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t1 = ((s1.x() - origin.x()) * d2y - (s1.y() - origin.y()) * d2x) / denominator;
        let t2 = ((s1.x() - origin.x()) * d1y - (s1.y() - origin.y()) * d1x) / denominator;

        if t1 >= T::ZERO - tolerance && t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance {
            Some(Point2D::new(origin.x() + t1 * d1x, origin.y() + t1 * d1y))
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for Ray2D<T> {
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
// Ray2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: Ray の起点を返す
        let origin = self.origin_internal();
        if triangle.contains_point(&origin) {
            Some(origin)
        } else {
            let centroid = triangle.centroid();
            if self.contains_point(&centroid, tolerance) {
                Some(centroid)
            } else {
                None
            }
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, triangle: &Triangle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // 三角形の頂点が Ray 上にあるか確認
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
// Ray2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> Option<Self::Point> {
        let center = ellipse.center();

        if self.distance_to_point(&center) <= ellipse.semi_major() + tolerance {
            Some(center) // 簡易実装: 楕円の中心を返す
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, ellipse: &Ellipse2D<T>, _tolerance: T) -> Vec<Self::Point> {
        let center = ellipse.center();

        if self.contains_point(&center, T::EPSILON) {
            vec![center] // 簡易実装
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ray2D vs Ray2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Ray2D<T>, tolerance: T) -> Option<Self::Point> {
        let origin1 = self.origin_internal();
        let origin2 = other.origin_internal();
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

        let t1 =
            ((origin2.x() - origin1.x()) * d2y - (origin2.y() - origin1.y()) * d2x) / denominator;
        let t2 =
            ((origin2.x() - origin1.x()) * d1y - (origin2.y() - origin1.y()) * d1x) / denominator;

        if t1 >= T::ZERO - tolerance && t2 >= T::ZERO - tolerance {
            Some(Point2D::new(origin1.x() + t1 * d1x, origin1.y() + t1 * d1y))
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ray2D<T>> for Ray2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &Ray2D<T>, tolerance: T) -> Vec<Self::Point> {
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

impl<T: Scalar> SelfIntersection<T> for Ray2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // Ray は自己交差しない
    }
}
