//! Ray2D Collision 実装
//!
//! BasicCollision トレイトの実装

use crate::{Arc2D, Circle2D, Ellipse2D, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D};
use geo_contracts::BasicCollision;
use geo_foundation::{Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar};

// ============================================================================
// Ray2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    fn overlaps(&self, _point: &Point2D<T>, _tolerance: T) -> bool {
        false // 点は重なりを持たない
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        self.distance_to_point(point)
    }
}

// ============================================================================
// Ray2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = Point2D::new(circle.center().0, circle.center().1);
        self.distance_to_point(&center) <= circle.radius() + tolerance
    }

    fn overlaps(&self, _circle: &Circle2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let dist = self.distance_to_point(&center);
        (dist - circle.radius()).max(T::ZERO)
    }
}

// ============================================================================
// Ray2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        self.distance_to_point(&center) <= arc.radius() + tolerance
    }

    fn overlaps(&self, _arc: &Arc2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, arc: &Arc2D<T>) -> T {
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        let dist = self.distance_to_point(&center);
        (dist - arc.radius()).max(T::ZERO)
    }
}

// ============================================================================
// Ray2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        let dist_start = self.distance_to_point(&start);
        let dist_end = self.distance_to_point(&end);

        dist_start <= tolerance || dist_end <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        self.contains_point(&start, tolerance) && self.contains_point(&end, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment2D<T>) -> T {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        let dist_start = self.distance_to_point(&start);
        let dist_end = self.distance_to_point(&end);

        dist_start.min(dist_end)
    }
}

// ============================================================================
// Ray2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        // Ray の起点が三角形内にある
        if triangle.contains_point(&self.origin_internal()) {
            return true;
        }

        // 三角形の頂点が Ray 上にある
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        if self.contains_point(&va, tolerance)
            || self.contains_point(&vb, tolerance)
            || self.contains_point(&vc, tolerance)
        {
            return true;
        }

        // 簡易実装: 三角形の重心が Ray の近傍にあるか
        let centroid = triangle.centroid();
        self.distance_to_point(&centroid) <= tolerance
    }

    fn overlaps(&self, _triangle: &Triangle2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, triangle: &Triangle2D<T>) -> T {
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        let dist_a = self.distance_to_point(&va);
        let dist_b = self.distance_to_point(&vb);
        let dist_c = self.distance_to_point(&vc);

        dist_a.min(dist_b).min(dist_c)
    }
}

// ============================================================================
// Ray2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> bool {
        let center = ellipse.center_internal();
        self.distance_to_point(&center) <= ellipse.semi_major_internal() + tolerance
    }

    fn overlaps(&self, _ellipse: &Ellipse2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, ellipse: &Ellipse2D<T>) -> T {
        ellipse.distance_to_point(&self.origin_internal())
    }
}

// ============================================================================
// Ray2D vs Ray2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray2D<T>> for Ray2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Ray2D<T>, tolerance: T) -> bool {
        // 簡易実装: 起点間の距離をチェック
        let dist = self.origin_internal().distance_to(&other.origin_internal());
        dist <= tolerance
    }

    fn overlaps(&self, other: &Ray2D<T>, tolerance: T) -> bool {
        // 起点が一致し、方向が同じ
        let origin_dist = self.origin_internal().distance_to(&other.origin_internal());
        let dir_self = self.direction_internal();
        let dir_other = other.direction_internal();

        let dir_diff = Vector2D::new(dir_self.x() - dir_other.x(), dir_self.y() - dir_other.y());

        origin_dist <= tolerance && dir_diff.length() <= tolerance
    }

    fn distance_to(&self, other: &Ray2D<T>) -> T {
        let dist1 = self.distance_to_point(&other.origin_internal());
        let dist2 = other.distance_to_point(&self.origin_internal());
        dist1.min(dist2)
    }
}
