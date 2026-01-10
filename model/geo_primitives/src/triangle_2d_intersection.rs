//! Triangle2D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Triangle2D, Vector2D};
use geo_foundation::{
    core::arc_traits::Arc2DProperties,
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Circle2DProperties, LineSegment2DProperties, Scalar,
};

// ============================================================================
// Triangle2D vs Point2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point2D<T>> for Triangle2D<T> {
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
// Triangle2D vs Circle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, circle: &Circle2D<T>, tolerance: T) -> Option<Self::Point> {
        let center = Point2D::new(circle.center().0, circle.center().1);

        if self.contains_point(&center) {
            return Some(center);
        }

        if self.distance_to_point(&center) <= circle.radius() + tolerance {
            Some(center) // 簡易実装: 円の中心を返す
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, circle: &Circle2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();
        let center = Point2D::new(circle.center().0, circle.center().1);

        // 簡易実装: 各辺と円の交点を計算
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        intersections.extend(self.edge_circle_intersections(
            va,
            vb,
            &center,
            circle.radius(),
            tolerance,
        ));
        intersections.extend(self.edge_circle_intersections(
            vb,
            vc,
            &center,
            circle.radius(),
            tolerance,
        ));
        intersections.extend(self.edge_circle_intersections(
            vc,
            va,
            &center,
            circle.radius(),
            tolerance,
        ));

        intersections
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 線分と円の交点を計算（ヘルパーメソッド）
    fn edge_circle_intersections(
        &self,
        p1: Point2D<T>,
        p2: Point2D<T>,
        center: &Point2D<T>,
        radius: T,
        tolerance: T,
    ) -> Vec<Point2D<T>> {
        let d = Vector2D::from_points(p1, p2);
        let f = Vector2D::from_points(*center, p1);

        let a = d.dot(&d);
        let b = (f.dot(&d)) * (T::ONE + T::ONE);
        let c = f.dot(&f) - radius * radius;

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();
        let two_a = (T::ONE + T::ONE) * a;

        let t1 = (-b - sqrt_disc) / two_a;
        let t2 = (-b + sqrt_disc) / two_a;

        let mut intersections = Vec::new();

        if t1 >= T::ZERO - tolerance && t1 <= T::ONE + tolerance {
            let point = Point2D::new(p1.x() + t1 * d.x(), p1.y() + t1 * d.y());
            intersections.push(point);
        }

        if t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance && (t2 - t1).abs() > tolerance {
            let point = Point2D::new(p1.x() + t2 * d.x(), p1.y() + t2 * d.y());
            intersections.push(point);
        }

        intersections
    }
}

// ============================================================================
// Triangle2D vs Arc2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc2D<T>> for Triangle2D<T> {
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

impl<T: Scalar> MultipleIntersection<T, Arc2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, arc: &Arc2D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: 空のベクタを返す
        let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
        let center = Point2D::new(center_x, center_y);

        if self.contains_point(&center) {
            vec![center]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Triangle2D vs LineSegment2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Option<Self::Point> {
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        // 各辺との交点を確認
        if let Some(p) = self.edge_segment_intersection(va, vb, segment, tolerance) {
            return Some(p);
        }
        if let Some(p) = self.edge_segment_intersection(vb, vc, segment, tolerance) {
            return Some(p);
        }
        if let Some(p) = self.edge_segment_intersection(vc, va, segment, tolerance) {
            return Some(p);
        }

        None
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, segment: &LineSegment2D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        if let Some(p) = self.edge_segment_intersection(va, vb, segment, tolerance) {
            intersections.push(p);
        }
        if let Some(p) = self.edge_segment_intersection(vb, vc, segment, tolerance) {
            intersections.push(p);
        }
        if let Some(p) = self.edge_segment_intersection(vc, va, segment, tolerance) {
            intersections.push(p);
        }

        intersections
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 三角形の辺と線分の交点を計算（ヘルパーメソッド）
    fn edge_segment_intersection(
        &self,
        edge_p1: Point2D<T>,
        edge_p2: Point2D<T>,
        segment: &LineSegment2D<T>,
        _tolerance: T,
    ) -> Option<Point2D<T>> {
        let s1 = Point2D::new(segment.start().0, segment.start().1);
        let s2 = Point2D::new(segment.end().0, segment.end().1);

        let d1 = Vector2D::from_points(s1, s2);
        let d2 = Vector2D::from_points(edge_p1, edge_p2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t1 = ((edge_p1.x() - s1.x()) * d2y - (edge_p1.y() - s1.y()) * d2x) / denominator;
        let t2 = ((edge_p1.x() - s1.x()) * d1y - (edge_p1.y() - s1.y()) * d1x) / denominator;

        if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
            Some(Point2D::new(s1.x() + t1 * d1x, s1.y() + t1 * d1y))
        } else {
            None
        }
    }
}

// ============================================================================
// Triangle2D vs Triangle2D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersection_with(&self, other: &Triangle2D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 重心を返す
        let centroid = self.centroid();
        if other.contains_point(&centroid) {
            Some(centroid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle2D<T>> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn intersections_with(&self, other: &Triangle2D<T>, _tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // 各辺の組み合わせで交点を計算
        let va = self.vertex_a_internal();
        let vb = self.vertex_b_internal();
        let vc = self.vertex_c_internal();

        let ova = other.vertex_a_internal();
        let ovb = other.vertex_b_internal();
        let ovc = other.vertex_c_internal();

        let edges_self = [(va, vb), (vb, vc), (vc, va)];
        let edges_other = [(ova, ovb), (ovb, ovc), (ovc, ova)];

        for &(p1, p2) in &edges_self {
            for &(q1, q2) in &edges_other {
                if let Some(point) = self.calculate_edge_intersection(p1, p2, q1, q2) {
                    intersections.push(point);
                }
            }
        }

        intersections
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 2つの辺の交点を計算（ヘルパーメソッド）
    fn calculate_edge_intersection(
        &self,
        p1: Point2D<T>,
        p2: Point2D<T>,
        q1: Point2D<T>,
        q2: Point2D<T>,
    ) -> Option<Point2D<T>> {
        let d1 = Vector2D::from_points(p1, p2);
        let d2 = Vector2D::from_points(q1, q2);

        let d1x = d1.x();
        let d1y = d1.y();
        let d2x = d2.x();
        let d2y = d2.y();

        let denominator = d1x * d2y - d1y * d2x;

        if denominator.abs() < T::EPSILON {
            return None; // 平行
        }

        let t1 = ((q1.x() - p1.x()) * d2y - (q1.y() - p1.y()) * d2x) / denominator;
        let t2 = ((q1.x() - p1.x()) * d1y - (q1.y() - p1.y()) * d1x) / denominator;

        if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
            Some(Point2D::new(p1.x() + t1 * d1x, p1.y() + t1 * d1y))
        } else {
            None
        }
    }
}

// ============================================================================
// SelfIntersection
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Triangle2D<T> {
    type Point = Point2D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 三角形は自己交差しない
    }
}
