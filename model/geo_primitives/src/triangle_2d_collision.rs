//! Triangle2D Collision 実装
//!
//! BasicCollision トレイトの実装

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Triangle2D, Vector2D};
use geo_foundation::{
    core::arc_core_traits::Arc2DProperties, extensions::BasicCollision, Circle2DProperties,
    LineSegment2DProperties, Scalar,
};

// ============================================================================
// Triangle2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point2D<T>> for Triangle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, point: &Point2D<T>, tolerance: T) -> bool {
        if self.contains_point(point) {
            return true;
        }
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
// Triangle2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Triangle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        // 円の中心が三角形内部にある場合
        let center = Point2D::new(circle.center().0, circle.center().1);
        if self.contains_point(&center) {
            return true;
        }

        // 三角形から円の中心までの距離が半径以下
        self.distance_to_point(&center) <= circle.radius() + tolerance
    }

    fn overlaps(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = Point2D::new(circle.center().0, circle.center().1);

        // 全ての頂点が円内にあるか確認
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        let dist_a = Vector2D::from_points(center, va).length();
        let dist_b = Vector2D::from_points(center, vb).length();
        let dist_c = Vector2D::from_points(center, vc).length();

        let r = circle.radius() + tolerance;
        dist_a <= r && dist_b <= r && dist_c <= r
    }

    fn distance_to(&self, circle: &Circle2D<T>) -> T {
        let center = Point2D::new(circle.center().0, circle.center().1);
        let dist = self.distance_to_point(&center);
        (dist - circle.radius()).max(T::ZERO)
    }
}

// ============================================================================
// Triangle2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc2D<T>> for Triangle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底円との距離チェック
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
// Triangle2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment2D<T>> for Triangle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        // 線分の端点が三角形内部にある
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        if self.contains_point(&start) || self.contains_point(&end) {
            return true;
        }

        // 三角形の各辺と線分が交差するか確認
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        self.segment_intersects_edge(segment, va, vb, tolerance)
            || self.segment_intersects_edge(segment, vb, vc, tolerance)
            || self.segment_intersects_edge(segment, vc, va, tolerance)
    }

    fn overlaps(&self, segment: &LineSegment2D<T>, tolerance: T) -> bool {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        self.contains_point(&start) && self.distance_to_point(&end) <= tolerance
    }

    fn distance_to(&self, segment: &LineSegment2D<T>) -> T {
        let start = Point2D::new(segment.start().0, segment.start().1);
        let end = Point2D::new(segment.end().0, segment.end().1);

        let dist_start = self.distance_to_point(&start);
        let dist_end = self.distance_to_point(&end);

        dist_start.min(dist_end)
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 線分と三角形の辺が交差するか確認（ヘルパーメソッド）
    fn segment_intersects_edge(
        &self,
        segment: &LineSegment2D<T>,
        edge_p1: Point2D<T>,
        edge_p2: Point2D<T>,
        tolerance: T,
    ) -> bool {
        let s1 = Point2D::new(segment.start().0, segment.start().1);
        let s2 = Point2D::new(segment.end().0, segment.end().1);

        // 線分同士の交差判定
        let d1 = Vector2D::from_points(s1, s2);
        let d2 = Vector2D::from_points(edge_p1, edge_p2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return false; // 平行
        }

        let t1 = ((edge_p1.x() - s1.x()) * d2y - (edge_p1.y() - s1.y()) * d2x) / denominator;
        let t2 = ((edge_p1.x() - s1.x()) * d1y - (edge_p1.y() - s1.y()) * d1x) / denominator;

        let lower = T::ZERO - tolerance;
        let upper = T::ONE + tolerance;

        t1 >= lower && t1 <= upper && t2 >= lower && t2 <= upper
    }
}

// ============================================================================
// Triangle2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle2D<T>> for Triangle2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Triangle2D<T>, tolerance: T) -> bool {
        // いずれかの頂点が他方の内部にある
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        if other.contains_point(&va) || other.contains_point(&vb) || other.contains_point(&vc) {
            return true;
        }

        let ova = other.vertex_a_internal();
        let ovb = other.vertex_b_internal();
        let ovc = other.vertex_c_internal();

        if self.contains_point(&ova) || self.contains_point(&ovb) || self.contains_point(&ovc) {
            return true;
        }

        // 辺同士の交差チェック
        let edges_self = [(va, vb), (vb, vc), (vc, va)];
        let edges_other = [(ova, ovb), (ovb, ovc), (ovc, ova)];

        for &(p1, p2) in &edges_self {
            for &(q1, q2) in &edges_other {
                if self.edges_intersect(p1, p2, q1, q2, tolerance) {
                    return true;
                }
            }
        }

        false
    }

    fn overlaps(&self, other: &Triangle2D<T>, tolerance: T) -> bool {
        // 全ての頂点が他方の内部にある
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        other.contains_point(&va)
            && (other.distance_to_point(&vb) <= tolerance)
            && (other.distance_to_point(&vc) <= tolerance)
    }

    fn distance_to(&self, other: &Triangle2D<T>) -> T {
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        let dist_a = other.distance_to_point(&va);
        let dist_b = other.distance_to_point(&vb);
        let dist_c = other.distance_to_point(&vc);

        dist_a.min(dist_b).min(dist_c)
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 2つの辺が交差するか判定（ヘルパーメソッド）
    fn edges_intersect(
        &self,
        p1: Point2D<T>,
        p2: Point2D<T>,
        q1: Point2D<T>,
        q2: Point2D<T>,
        tolerance: T,
    ) -> bool {
        let d1 = Vector2D::from_points(p1, p2);
        let d2 = Vector2D::from_points(q1, q2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return false; // 平行
        }

        let t1 = ((q1.x() - p1.x()) * d2y - (q1.y() - p1.y()) * d2x) / denominator;
        let t2 = ((q1.x() - p1.x()) * d1y - (q1.y() - p1.y()) * d1x) / denominator;

        let lower = T::ZERO - tolerance;
        let upper = T::ONE + tolerance;

        t1 >= lower && t1 <= upper && t2 >= lower && t2 <= upper
    }
}
