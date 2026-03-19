//! InfiniteLine2D Collision 実装
//!
//! BasicCollision トレイトの実装

use crate::{
    Arc2D, Circle2D, Ellipse2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D,
};
use geo_contracts::BasicCollision;
use geo_contracts::{Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar};

// ============================================================================
// InfiniteLine2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point2D<T>> for InfiniteLine2D<T> {
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
// InfiniteLine2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle2D<T>> for InfiniteLine2D<T> {
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
// InfiniteLine2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc2D<T>> for InfiniteLine2D<T> {
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
// InfiniteLine2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for InfiniteLine2D<T> {
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
// InfiniteLine2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle2D<T>> for InfiniteLine2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, triangle: &Triangle2D<T>, tolerance: T) -> bool {
        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        // いずれかの頂点が直線上にある
        if self.contains_point(&va, tolerance)
            || self.contains_point(&vb, tolerance)
            || self.contains_point(&vc, tolerance)
        {
            return true;
        }

        // 頂点が直線の両側にある場合、交差している
        let normal = self.normal_internal();
        let to_a = Vector2D::from_points(self.point_internal(), va);
        let to_b = Vector2D::from_points(self.point_internal(), vb);
        let to_c = Vector2D::from_points(self.point_internal(), vc);

        let side_a = to_a.dot(&normal);
        let side_b = to_b.dot(&normal);
        let side_c = to_c.dot(&normal);

        // 異なる符号がある場合、直線が三角形を横切る
        (side_a * side_b < T::ZERO) || (side_b * side_c < T::ZERO) || (side_c * side_a < T::ZERO)
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
// InfiniteLine2D vs Ellipse2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse2D<T>> for InfiniteLine2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, ellipse: &Ellipse2D<T>, tolerance: T) -> bool {
        let center = ellipse.center_internal();
        self.distance_to_point(&center) <= ellipse.semi_major_internal() + tolerance
    }

    fn overlaps(&self, _ellipse: &Ellipse2D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, ellipse: &Ellipse2D<T>) -> T {
        ellipse.distance_to_point(&self.point_internal())
    }
}

// ============================================================================
// InfiniteLine2D vs Ray2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray2D<T>> for InfiniteLine2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, ray: &Ray2D<T>, tolerance: T) -> bool {
        let origin = ray.origin_internal();
        self.contains_point(&origin, tolerance)
    }

    fn overlaps(&self, ray: &Ray2D<T>, tolerance: T) -> bool {
        // Ray の起点が直線上にあり、方向が一致
        let origin = ray.origin_internal();
        if !self.contains_point(&origin, tolerance) {
            return false;
        }

        let ray_dir = ray.direction_internal();
        let line_dir = self.direction_internal();

        let dir_diff = Vector2D::new(ray_dir.x() - line_dir.x(), ray_dir.y() - line_dir.y());

        dir_diff.length() <= tolerance
    }

    fn distance_to(&self, ray: &Ray2D<T>) -> T {
        let origin = ray.origin_internal();
        self.distance_to_point(&origin)
    }
}

// ============================================================================
// InfiniteLine2D vs InfiniteLine2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine2D<T>> for InfiniteLine2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &InfiniteLine2D<T>, tolerance: T) -> bool {
        // 平行でなければ必ず交わる
        let dir1 = self.direction_internal();
        let dir2 = other.direction_internal();

        !dir1.is_parallel(&dir2, tolerance)
    }

    fn overlaps(&self, other: &InfiniteLine2D<T>, tolerance: T) -> bool {
        // 平行かつ距離がゼロ（同一直線）
        let dir1 = self.direction_internal();
        let dir2 = other.direction_internal();

        if !dir1.is_parallel(&dir2, tolerance) {
            return false;
        }

        let point_on_other = other.point_internal();
        self.contains_point(&point_on_other, tolerance)
    }

    fn distance_to(&self, other: &InfiniteLine2D<T>) -> T {
        let dir1 = self.direction_internal();
        let dir2 = other.direction_internal();

        // 平行な場合のみ距離が定義される
        if dir1.is_parallel(&dir2, T::EPSILON) {
            let point_on_other = other.point_internal();
            self.distance_to_point(&point_on_other)
        } else {
            T::ZERO // 交差する場合
        }
    }
}
