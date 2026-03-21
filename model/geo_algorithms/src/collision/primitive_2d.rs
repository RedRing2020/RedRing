//! 2D Primitive collision algorithms
//!
//! Phase C Step 1: `geo_primitives` から 2D 衝突判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。
//!
//! 注意: orphan rules により、ここでは trait 実装ではなく
//! 形状ペア関数を提供する。

use crate::intersection::pair_base::line_segment2d_arc2d_intersections;
use crate::intersection::primitive_2d::triangle2d_line_segment2d_intersections;
use crate::{
    Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D,
    Triangle2D, Vector2D,
};
use geo_contracts::{
    Arc2DProperties, BasicCollision, Circle2DProperties, LineSegment2DProperties, Scalar,
    Triangle2DProperties,
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

pub fn circle2d_arc2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> bool {
    arc2d_circle2d_collides(arc, circle, tolerance)
}

pub fn line_segment2d_arc2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
) -> bool {
    !line_segment2d_arc2d_intersections(segment, arc).is_empty()
}

pub fn arc2d_line_segment2d_collides<T: Scalar>(
    arc: &Arc2D<T>,
    segment: &LineSegment2D<T>,
) -> bool {
    line_segment2d_arc2d_collides(segment, arc)
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

pub fn circle2d_ray2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    ray2d_circle2d_collides(ray, circle, tolerance)
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

pub fn line_segment2d_ray2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    ray2d_line_segment2d_collides(ray, segment, tolerance)
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

pub fn circle2d_triangle2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    triangle: &Triangle2D<T>,
    tolerance: T,
) -> bool {
    triangle2d_circle2d_collides(triangle, circle, tolerance)
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

pub fn triangle2d_line_segment2d_collides<T: Scalar>(
    triangle: &Triangle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    !triangle2d_line_segment2d_intersections(triangle, segment, tolerance).is_empty()
}

pub fn line_segment2d_triangle2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    triangle: &Triangle2D<T>,
    tolerance: T,
) -> bool {
    triangle2d_line_segment2d_collides(triangle, segment, tolerance)
}

pub fn infinite_line2d_point2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> bool {
    line.intersects(point, tolerance)
}

pub fn infinite_line2d_circle2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    line.intersects(circle, tolerance)
}

pub fn circle2d_infinite_line2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_circle2d_collides(line, circle, tolerance)
}

pub fn infinite_line2d_line_segment2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    line.intersects(segment, tolerance)
}

pub fn line_segment2d_infinite_line2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_line_segment2d_collides(line, segment, tolerance)
}

pub fn infinite_line2d_ray2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    line.intersects(ray, tolerance)
}

pub fn ray2d_infinite_line2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_ray2d_collides(line, ray, tolerance)
}

pub fn ellipse2d_point2d_collides<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(point, tolerance)
}

pub fn ellipse2d_circle2d_collides<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(circle, tolerance)
}

pub fn circle2d_ellipse2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> bool {
    ellipse2d_circle2d_collides(ellipse, circle, tolerance)
}

pub fn ellipse_arc2d_point2d_collides<T: Scalar>(
    arc: &EllipseArc2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> bool {
    arc.contains_point(point, tolerance)
}

pub fn ellipse_arc2d_circle2d_collides<T: Scalar>(
    arc: &EllipseArc2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> bool {
    if !arc.ellipse().intersects(circle, tolerance) {
        return false;
    }

    let center = Point2D::new(circle.center().0, circle.center().1);
    if arc.point_in_angle_range(&center, tolerance) {
        return true;
    }

    let start = arc.start_point();
    let end = arc.end_point();
    circle2d_point2d_collides(circle, &start, tolerance)
        || circle2d_point2d_collides(circle, &end, tolerance)
}

pub fn circle2d_ellipse_arc2d_collides<T: Scalar>(
    circle: &Circle2D<T>,
    arc: &EllipseArc2D<T>,
    tolerance: T,
) -> bool {
    ellipse_arc2d_circle2d_collides(arc, circle, tolerance)
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

#[cfg(test)]
mod tests {
    use super::{
        arc2d_line_segment2d_collides, circle2d_arc2d_collides, circle2d_circle2d_collides,
        circle2d_ellipse2d_collides, circle2d_ellipse_arc2d_collides,
        circle2d_infinite_line2d_collides, circle2d_point2d_collides, circle2d_ray2d_collides,
        circle2d_triangle2d_collides, ellipse2d_circle2d_collides, ellipse2d_point2d_collides,
        ellipse_arc2d_circle2d_collides, ellipse_arc2d_point2d_collides,
        infinite_line2d_circle2d_collides, infinite_line2d_line_segment2d_collides,
        infinite_line2d_ray2d_collides, line_segment2d_arc2d_collides,
        line_segment2d_circle2d_collides, line_segment2d_infinite_line2d_collides,
        line_segment2d_ray2d_collides, line_segment2d_triangle2d_collides, ray2d_circle2d_collides,
        ray2d_infinite_line2d_collides, ray2d_line_segment2d_collides,
        triangle2d_circle2d_collides, triangle2d_line_segment2d_collides,
        triangle2d_triangle2d_collides,
    };
    use crate::{
        Angle, Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, LineSegment2D, Point2D,
        Ray2D, Triangle2D, Vector2D,
    };

    #[test]
    fn circle_point_collision_detects_boundary_point() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let point = Point2D::new(1.0, 0.0);

        assert!(circle2d_point2d_collides(&circle, &point, 1e-9));
    }

    #[test]
    fn circle_circle_collision_detects_overlap() {
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(1.5, 0.0), 1.0).unwrap();

        assert!(circle2d_circle2d_collides(&circle1, &circle2, 1e-9));
    }

    #[test]
    fn line_segment_circle_collision_detects_crossing_segment() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        assert!(line_segment2d_circle2d_collides(&segment, &circle, 1e-9));
    }

    #[test]
    fn triangle_triangle_collision_detects_edge_crossing() {
        let triangle1 = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();
        let triangle2 = Triangle2D::new(
            Point2D::new(1.0, -1.0),
            Point2D::new(3.0, 1.0),
            Point2D::new(1.0, 1.0),
        )
        .unwrap();

        assert!(triangle2d_triangle2d_collides(&triangle1, &triangle2, 1e-9));
    }

    #[test]
    fn line_segment_arc_collision_detects_crossing_segment() {
        let arc = Arc2D::new(
            Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap(),
            crate::Angle::from_degrees(0.0),
            crate::Angle::from_degrees(180.0),
        )
        .unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        assert!(line_segment2d_arc2d_collides(&segment, &arc));
        assert!(circle2d_arc2d_collides(
            &Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap(),
            &arc,
            1e-9
        ));
    }

    #[test]
    fn triangle_line_segment_collision_detects_intersection() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();
        let segment = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

        assert!(triangle2d_line_segment2d_collides(
            &triangle, &segment, 1e-9
        ));
    }

    #[test]
    fn symmetric_collision_wrappers_match_base_functions() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();
        let arc = Arc2D::new(
            Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap(),
            crate::Angle::from_degrees(0.0),
            crate::Angle::from_degrees(180.0),
        )
        .unwrap();

        let tol = 1e-9;
        assert_eq!(
            circle2d_ray2d_collides(&circle, &ray, tol),
            ray2d_circle2d_collides(&ray, &circle, tol)
        );
        assert_eq!(
            line_segment2d_ray2d_collides(&segment, &ray, tol),
            ray2d_line_segment2d_collides(&ray, &segment, tol)
        );
        assert_eq!(
            circle2d_triangle2d_collides(&circle, &triangle, tol),
            triangle2d_circle2d_collides(&triangle, &circle, tol)
        );
        assert_eq!(
            line_segment2d_triangle2d_collides(&segment, &triangle, tol),
            triangle2d_line_segment2d_collides(&triangle, &segment, tol)
        );
        assert_eq!(
            arc2d_line_segment2d_collides(&arc, &segment),
            line_segment2d_arc2d_collides(&segment, &arc)
        );

        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        assert_eq!(
            circle2d_infinite_line2d_collides(&circle, &line, tol),
            infinite_line2d_circle2d_collides(&line, &circle, tol)
        );
        assert_eq!(
            line_segment2d_infinite_line2d_collides(&segment, &line, tol),
            infinite_line2d_line_segment2d_collides(&line, &segment, tol)
        );
        assert_eq!(
            ray2d_infinite_line2d_collides(&ray, &line, tol),
            infinite_line2d_ray2d_collides(&line, &ray, tol)
        );
    }

    #[test]
    fn ellipse_and_ellipse_arc_collision_entry_points_work() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 3.0, 2.0, 0.0).unwrap();
        let ellipse_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_degrees(0.0),
            Angle::from_degrees(180.0),
        );
        let point = Point2D::new(3.0, 0.0);
        let circle = Circle2D::new(Point2D::new(2.5, 0.0), 0.75).unwrap();

        assert!(ellipse2d_point2d_collides(&ellipse, &point, 1e-6));
        assert!(ellipse_arc2d_point2d_collides(&ellipse_arc, &point, 1e-6));
        assert_eq!(
            circle2d_ellipse2d_collides(&circle, &ellipse, 1e-6),
            ellipse2d_circle2d_collides(&ellipse, &circle, 1e-6)
        );
        assert_eq!(
            circle2d_ellipse_arc2d_collides(&circle, &ellipse_arc, 1e-6),
            ellipse_arc2d_circle2d_collides(&ellipse_arc, &circle, 1e-6)
        );
    }
}
