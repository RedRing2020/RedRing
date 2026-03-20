//! 2D Primitive collision algorithms
//!
//! Phase C Step 1: `geo_primitives` から 2D 衝突判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。
//!
//! 注意: orphan rules により、ここでは trait 実装ではなく
//! 形状ペア関数を提供する。

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D};
use geo_contracts::{
    Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar, Triangle2DProperties,
};

pub fn circle2d_point2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> bool {
    let dx = point.x() - circle.center().0;
    let dy = point.y() - circle.center().1;
    let distance = (dx * dx + dy * dy).sqrt();
    (distance - circle.radius()).abs() <= tolerance
}

pub fn circle2d_circle2d_collides<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
    tolerance: T,
) -> bool {
    let dx = circle2.center().0 - circle1.center().0;
    let dy = circle2.center().1 - circle1.center().1;
    let center_distance = (dx * dx + dy * dy).sqrt();

    let radii_sum = circle1.radius() + circle2.radius();
    let radii_diff = (circle1.radius() - circle2.radius()).abs();

    center_distance <= radii_sum + tolerance && center_distance >= radii_diff - tolerance
}

pub fn line_segment2d_point2d_distance<T: Scalar>(
    segment: &LineSegment2D<T>,
    point: &Point2D<T>,
) -> T {
    let start = segment.start();
    let end = segment.end();

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length_sq = dx * dx + dy * dy;

    if length_sq == T::ZERO {
        let dist_x = point.x() - start.0;
        let dist_y = point.y() - start.1;
        return (dist_x * dist_x + dist_y * dist_y).sqrt();
    }

    let t = {
        let dot = (point.x() - start.0) * dx + (point.y() - start.1) * dy;
        (dot / length_sq).max(T::ZERO).min(T::ONE)
    };

    let closest_x = start.0 + t * dx;
    let closest_y = start.1 + t * dy;

    let dist_x = point.x() - closest_x;
    let dist_y = point.y() - closest_y;
    (dist_x * dist_x + dist_y * dist_y).sqrt()
}

pub fn line_segment2d_circle2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let distance_to_center = line_segment2d_point2d_distance(segment, &center);
    (distance_to_center - circle.radius()).max(T::ZERO) <= tolerance
}

pub fn arc2d_point2d_collides<T: Scalar>(arc: &Arc2D<T>, point: &Point2D<T>, tolerance: T) -> bool {
    let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let dx = point.x() - center_x;
    let dy = point.y() - center_y;
    let distance = (dx * dx + dy * dy).sqrt();

    (distance - arc.radius()).abs() <= tolerance
}

pub fn arc2d_circle2d_collides<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    let (center1_x, center1_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let (center2_x, center2_y) = circle.center();

    let dx = center2_x - center1_x;
    let dy = center2_y - center1_y;
    let center_distance = (dx * dx + dy * dy).sqrt();

    let radii_sum = arc.radius() + circle.radius();
    let radii_diff = (arc.radius() - circle.radius()).abs();

    center_distance <= radii_sum + tolerance && center_distance >= radii_diff - tolerance
}

pub fn ray2d_point2d_collides<T: Scalar>(ray: &Ray2D<T>, point: &Point2D<T>, tolerance: T) -> bool {
    ray.contains_point(point, tolerance)
}

pub fn ray2d_circle2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    let center = Point2D::new(circle.center().0, circle.center().1);
    ray.distance_to_point(&center) <= circle.radius() + tolerance
}

pub fn ray2d_line_segment2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    let start = Point2D::new(segment.start().0, segment.start().1);
    let end = Point2D::new(segment.end().0, segment.end().1);

    let dist_start = ray.distance_to_point(&start);
    let dist_end = ray.distance_to_point(&end);

    dist_start <= tolerance || dist_end <= tolerance
}

pub fn triangle2d_circle2d_collides<T: Scalar>(
    triangle: &Triangle2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    let center = Point2D::new(circle.center().0, circle.center().1);
    if triangle.contains_point(&center) {
        return true;
    }

    triangle.distance_to_point(&center) <= circle.radius() + tolerance
}

pub fn triangle2d_triangle2d_collides<T: Scalar>(
    triangle1: &Triangle2D<T>,
    triangle2: &Triangle2D<T>,
    tolerance: T,
) -> bool {
    let va_tuple = triangle1.vertex_a();
    let vb_tuple = triangle1.vertex_b();
    let vc_tuple = triangle1.vertex_c();
    let va = Point2D::new(va_tuple.0, va_tuple.1);
    let vb = Point2D::new(vb_tuple.0, vb_tuple.1);
    let vc = Point2D::new(vc_tuple.0, vc_tuple.1);

    if triangle2.contains_point(&va)
        || triangle2.contains_point(&vb)
        || triangle2.contains_point(&vc)
    {
        return true;
    }

    let ova_tuple = triangle2.vertex_a();
    let ovb_tuple = triangle2.vertex_b();
    let ovc_tuple = triangle2.vertex_c();
    let ova = Point2D::new(ova_tuple.0, ova_tuple.1);
    let ovb = Point2D::new(ovb_tuple.0, ovb_tuple.1);
    let ovc = Point2D::new(ovc_tuple.0, ovc_tuple.1);

    if triangle1.contains_point(&ova)
        || triangle1.contains_point(&ovb)
        || triangle1.contains_point(&ovc)
    {
        return true;
    }

    let edges_self = [(va, vb), (vb, vc), (vc, va)];
    let edges_other = [(ova, ovb), (ovb, ovc), (ovc, ova)];

    for &(p1, p2) in &edges_self {
        for &(q1, q2) in &edges_other {
            if edges_intersect(p1, p2, q1, q2, tolerance) {
                return true;
            }
        }
    }

    false
}

fn edges_intersect<T: Scalar>(
    p1: Point2D<T>,
    p2: Point2D<T>,
    q1: Point2D<T>,
    q2: Point2D<T>,
    tolerance: T,
) -> bool {
    let d1 = Vector2D::from_points(p1, p2);
    let d2 = Vector2D::from_points(q1, q2);

    let denominator = d1.x() * d2.y() - d1.y() * d2.x();
    if denominator.abs() < T::EPSILON {
        return false;
    }

    let t1 = ((q1.x() - p1.x()) * d2.y() - (q1.y() - p1.y()) * d2.x()) / denominator;
    let t2 = ((q1.x() - p1.x()) * d1.y() - (q1.y() - p1.y()) * d1.x()) / denominator;

    let lower = T::ZERO - tolerance;
    let upper = T::ONE + tolerance;
    t1 >= lower && t1 <= upper && t2 >= lower && t2 <= upper
}
