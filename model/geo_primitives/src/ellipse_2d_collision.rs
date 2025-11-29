//! Ellipse2D Collision 実装
//!
//! BasicCollision トレイトの実装

use crate::{Arc2D, Circle2D, Ellipse2D, LineSegment2D, Point2D, Triangle2D, Vector2D};
use geo_foundation::{
    core::arc_core_traits::Arc2DProperties, extensions::BasicCollision, Circle2DProperties,
    LineSegment2DProperties, Scalar,
};

// ============================================================================
// Ellipse2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn overlaps(&self, _point: &Point2D<T>, _tolerance: T) -> bool {
        false // 点は重なりを持たない
    }

    fn distance_to(&self, point: &Point2D<T>) -> T {
        self.distance_to_point(point)
    }
}

// ============================================================================
// Ellipse2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let dist = self.distance_to_point(&center);
        dist <= circle.radius() + tolerance
    }

    fn overlaps(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = Point2D::new(circle.center().0, circle.center().1);

        // 簡易実装: 楕円の中心が円内にあるか確認
        let dist = Vector2D::from_points(self.center(), center).length();
        dist + self.semi_major() <= circle.radius() + tolerance
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let dist = self.distance_to_point(&center);
        (dist - circle.radius()).max(T::ZERO)
    }
}

// ============================================================================
// Ellipse2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        let dist = self.distance_to_point(&center);
        dist <= arc.radius() + tolerance
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
// Ellipse2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        // 線分の端点が楕円内にある
        if self.distance_to_point(&start) <= tolerance || self.distance_to_point(&end) <= tolerance
        {
            return true;
        }

        // 簡易実装: 線分の中点が楕円の近傍にあるか
        let mid = Point2D::new(
            (start.x() + end.x()) / (T::ONE + T::ONE),
            (start.y() + end.y()) / (T::ONE + T::ONE),
        );
        self.distance_to_point(&mid) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        self.distance_to_point(&start) <= tolerance && self.distance_to_point(&end) <= tolerance
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
// Ellipse2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        // 楕円の中心が三角形内にある
        if triangle.contains_point(&self.center()) {
            return true;
        }

        // 三角形の頂点が楕円内にある
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        if self.distance_to_point(&va) <= tolerance
            || self.distance_to_point(&vb) <= tolerance
            || self.distance_to_point(&vc) <= tolerance
        {
            return true;
        }

        // 簡易実装: 三角形の重心が楕円の近傍にあるか
        let centroid = triangle.centroid();
        self.distance_to_point(&centroid) <= tolerance
    }

    fn overlaps(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        self.distance_to_point(&va) <= tolerance
            && self.distance_to_point(&vb) <= tolerance
            && self.distance_to_point(&vc) <= tolerance
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
// Ellipse2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse2D<T>> for Ellipse2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Ellipse2D<T>, tolerance: T) -> bool {
        // 簡易実装: 中心間距離と長軸の和を比較
        let center_dist = Vector2D::from_points(self.center(), other.center()).length();
        let sum_semi_major = self.semi_major() + other.semi_major();

        center_dist <= sum_semi_major + tolerance
    }

    fn overlaps(&self, other: &Ellipse2D<T>, tolerance: T) -> bool {
        // 簡易実装: 一方の中心が他方の楕円内にある
        let dist_to_other = self.distance_to_point(&other.center());
        dist_to_other <= tolerance
    }

    fn distance_to(&self, other: &Ellipse2D<T>) -> T {
        // 簡易実装: 中心間距離から長軸を引く
        let center_dist = Vector2D::from_points(self.center(), other.center()).length();
        let radii_sum = self.semi_major() + other.semi_major();
        (center_dist - radii_sum).max(T::ZERO)
    }
}
