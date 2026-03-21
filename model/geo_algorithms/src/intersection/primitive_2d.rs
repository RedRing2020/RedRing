//! 2D Primitive intersection algorithms
//!
//! Phase C Step 1: `geo_primitives` から 2D 交差判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。

use crate::intersection::pair_base::{
    arc2d_circle2d_intersections, circle2d_circle2d_intersections,
    circle2d_line_segment2d_intersections, line_segment2d_arc2d_intersections,
    line_segment2d_circle2d_intersections, line_segment2d_line_segment2d_intersection,
};
use crate::{
    Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D,
    Triangle2D, Vector2D,
};
use geo_contracts::{
    Arc2DProperties, BasicIntersection, Circle2DProperties, Ellipse2DProperties,
    LineSegment2DProperties, MultipleIntersection, Ray2DProperties, Scalar, Triangle2DProperties,
};

pub fn circle2d_point2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let dx = point.x() - circle.center().0;
    let dy = point.y() - circle.center().1;
    let distance = (dx * dx + dy * dy).sqrt();

    if (distance - circle.radius()).abs() <= tolerance {
        Some(*point)
    } else {
        None
    }
}

pub fn circle2d_circle2d_intersections_algo<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    circle2d_circle2d_intersections(circle1, circle2)
}

pub fn circle2d_line_segment2d_intersections_algo<T: Scalar>(
    circle: &Circle2D<T>,
    segment: &LineSegment2D<T>,
) -> Vec<Point2D<T>> {
    circle2d_line_segment2d_intersections(circle, segment)
}

pub fn arc2d_circle2d_intersections_algo<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    arc2d_circle2d_intersections(arc, circle)
}

pub fn circle2d_arc2d_intersections_algo<T: Scalar>(
    circle: &Circle2D<T>,
    arc: &Arc2D<T>,
) -> Vec<Point2D<T>> {
    arc2d_circle2d_intersections(arc, circle)
}

pub fn line_segment2d_circle2d_intersections_algo<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    line_segment2d_circle2d_intersections(segment, circle)
}

pub fn line_segment2d_arc2d_intersections_algo<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
) -> Vec<Point2D<T>> {
    line_segment2d_arc2d_intersections(segment, arc)
}

pub fn arc2d_line_segment2d_intersections_algo<T: Scalar>(
    arc: &Arc2D<T>,
    segment: &LineSegment2D<T>,
) -> Vec<Point2D<T>> {
    line_segment2d_arc2d_intersections(segment, arc)
}

pub fn line_segment2d_line_segment2d_intersection_algo<T: Scalar>(
    segment1: &LineSegment2D<T>,
    segment2: &LineSegment2D<T>,
) -> Option<Point2D<T>> {
    line_segment2d_line_segment2d_intersection(segment1, segment2)
}

pub fn ray2d_line_segment2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let s1 = Point2D::new(segment.start().0, segment.start().1);
    let s2 = Point2D::new(segment.end().0, segment.end().1);

    let origin_tuple = ray.origin();
    let direction_tuple = ray.direction();
    let origin = Point2D::new(origin_tuple.0, origin_tuple.1);
    let direction = Vector2D::new(direction_tuple.0, direction_tuple.1);

    let d1 = direction;
    let d2 = Vector2D::from_points(s1, s2);

    let denominator = d1.x() * d2.y() - d1.y() * d2.x();
    if denominator.abs() < T::EPSILON {
        return None;
    }

    let t1 = ((s1.x() - origin.x()) * d2.y() - (s1.y() - origin.y()) * d2.x()) / denominator;
    let t2 = ((s1.x() - origin.x()) * d1.y() - (s1.y() - origin.y()) * d1.x()) / denominator;

    if t1 >= T::ZERO - tolerance && t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance {
        Some(Point2D::new(
            origin.x() + t1 * d1.x(),
            origin.y() + t1 * d1.y(),
        ))
    } else {
        None
    }
}

pub fn line_segment2d_ray2d_intersection<T: Scalar>(
    segment: &LineSegment2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    ray2d_line_segment2d_intersection(ray, segment, tolerance)
}

pub fn ray2d_circle2d_intersections<T: Scalar>(
    ray: &Ray2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let origin_tuple = ray.origin();
    let direction_tuple = ray.direction();
    let origin = Point2D::new(origin_tuple.0, origin_tuple.1);
    let direction = Vector2D::new(direction_tuple.0, direction_tuple.1);

    let oc = origin - center;
    let dir_vec = direction;

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
        intersections.push(Point2D::new(
            origin.x() + t1 * dir_vec.x(),
            origin.y() + t1 * dir_vec.y(),
        ));
    }

    if t2 >= T::ZERO - tolerance && (t2 - t1).abs() > tolerance {
        intersections.push(Point2D::new(
            origin.x() + t2 * dir_vec.x(),
            origin.y() + t2 * dir_vec.y(),
        ));
    }

    intersections
}

pub fn circle2d_ray2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    ray2d_circle2d_intersections(ray, circle, tolerance)
}

pub fn triangle2d_line_segment2d_intersections<T: Scalar>(
    triangle: &Triangle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut intersections = Vec::new();

    let va_tuple = triangle.vertex_a();
    let vb_tuple = triangle.vertex_b();
    let vc_tuple = triangle.vertex_c();
    let va = Point2D::new(va_tuple.0, va_tuple.1);
    let vb = Point2D::new(vb_tuple.0, vb_tuple.1);
    let vc = Point2D::new(vc_tuple.0, vc_tuple.1);

    if let Some(p) = edge_segment_intersection(va, vb, segment, tolerance) {
        intersections.push(p);
    }
    if let Some(p) = edge_segment_intersection(vb, vc, segment, tolerance) {
        intersections.push(p);
    }
    if let Some(p) = edge_segment_intersection(vc, va, segment, tolerance) {
        intersections.push(p);
    }

    intersections
}

pub fn ray2d_ellipse2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let center_tuple = ellipse.center();
    let center = Point2D::new(center_tuple.0, center_tuple.1);
    if ray.distance_to_point(&center) <= ellipse.semi_major_axis() + tolerance {
        Some(center)
    } else {
        None
    }
}

pub fn arc2d_point2d_intersection<T: Scalar>(
    arc: &Arc2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let dx = point.x() - center_x;
    let dy = point.y() - center_y;
    let distance = (dx * dx + dy * dy).sqrt();

    if (distance - arc.radius()).abs() > tolerance {
        return None;
    }

    if !arc.contains_point_angle(*point) {
        return None;
    }

    Some(*point)
}

pub fn infinite_line2d_point2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    line.intersection_with(point, tolerance)
}

pub fn infinite_line2d_circle2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    line.intersection_with(circle, tolerance)
}

pub fn infinite_line2d_circle2d_intersections<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    line.intersections_with(circle, tolerance)
}

pub fn circle2d_infinite_line2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    infinite_line2d_circle2d_intersection(line, circle, tolerance)
}

pub fn circle2d_infinite_line2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    infinite_line2d_circle2d_intersections(line, circle, tolerance)
}

pub fn infinite_line2d_line_segment2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    line.intersection_with(segment, tolerance)
}

pub fn line_segment2d_infinite_line2d_intersection<T: Scalar>(
    segment: &LineSegment2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    infinite_line2d_line_segment2d_intersection(line, segment, tolerance)
}

pub fn infinite_line2d_ray2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    line.intersection_with(ray, tolerance)
}

pub fn ray2d_infinite_line2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    infinite_line2d_ray2d_intersection(line, ray, tolerance)
}

pub fn ellipse2d_point2d_intersection<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    ellipse.intersection_with(point, tolerance)
}

pub fn ellipse2d_circle2d_intersection<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    ellipse.intersection_with(circle, tolerance)
}

pub fn ellipse2d_circle2d_intersections<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    ellipse.intersections_with(circle, tolerance)
}

pub fn circle2d_ellipse2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    ellipse2d_circle2d_intersection(ellipse, circle, tolerance)
}

pub fn circle2d_ellipse2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    ellipse2d_circle2d_intersections(ellipse, circle, tolerance)
}

pub fn ellipse_arc2d_point2d_intersection<T: Scalar>(
    arc: &EllipseArc2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    if arc.contains_point(point, tolerance) {
        Some(*point)
    } else {
        None
    }
}

fn edge_segment_intersection<T: Scalar>(
    edge_p1: Point2D<T>,
    edge_p2: Point2D<T>,
    segment: &LineSegment2D<T>,
    _tolerance: T,
) -> Option<Point2D<T>> {
    let s1 = Point2D::new(segment.start().0, segment.start().1);
    let s2 = Point2D::new(segment.end().0, segment.end().1);

    let d1 = Vector2D::from_points(s1, s2);
    let d2 = Vector2D::from_points(edge_p1, edge_p2);

    let denominator = d1.x() * d2.y() - d1.y() * d2.x();
    if denominator.abs() < T::EPSILON {
        return None;
    }

    let t1 = ((edge_p1.x() - s1.x()) * d2.y() - (edge_p1.y() - s1.y()) * d2.x()) / denominator;
    let t2 = ((edge_p1.x() - s1.x()) * d1.y() - (edge_p1.y() - s1.y()) * d1.x()) / denominator;

    if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
        Some(Point2D::new(s1.x() + t1 * d1.x(), s1.y() + t1 * d1.y()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        arc2d_line_segment2d_intersections_algo, circle2d_arc2d_intersections_algo,
        circle2d_circle2d_intersections_algo, circle2d_ellipse2d_intersection,
        circle2d_ellipse2d_intersections, circle2d_infinite_line2d_intersection,
        circle2d_infinite_line2d_intersections, circle2d_point2d_intersection,
        circle2d_ray2d_intersections, ellipse2d_circle2d_intersection,
        ellipse2d_circle2d_intersections, ellipse2d_point2d_intersection,
        ellipse_arc2d_point2d_intersection, infinite_line2d_circle2d_intersection,
        infinite_line2d_circle2d_intersections, infinite_line2d_line_segment2d_intersection,
        infinite_line2d_ray2d_intersection, line_segment2d_infinite_line2d_intersection,
        line_segment2d_line_segment2d_intersection_algo, line_segment2d_ray2d_intersection,
        ray2d_circle2d_intersections, ray2d_infinite_line2d_intersection,
        ray2d_line_segment2d_intersection,
    };
    use crate::{
        Angle, Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, LineSegment2D, Point2D,
        Ray2D, Vector2D,
    };

    #[test]
    fn circle_point_intersection_returns_same_point() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let point = Point2D::new(1.0, 0.0);

        let intersection = circle2d_point2d_intersection(&circle, &point, 1e-9);
        assert_eq!(intersection, Some(point));
    }

    #[test]
    fn circle_circle_intersection_returns_two_points() {
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap();

        let intersections = circle2d_circle2d_intersections_algo(&circle1, &circle2);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn line_segment_line_segment_intersection_returns_crossing_point() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let segment2 = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

        let intersection = line_segment2d_line_segment2d_intersection_algo(&segment1, &segment2);
        assert_eq!(intersection, Some(Point2D::new(1.0, 0.0)));
    }

    #[test]
    fn ray_circle_intersection_returns_forward_hits_only() {
        let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let intersections = ray2d_circle2d_intersections(&ray, &circle, 1e-9);
        assert_eq!(intersections.len(), 2);
        assert!(intersections.iter().all(|point| point.x() >= -1.0));
    }

    #[test]
    fn circle_arc_intersection_symmetric_entry_point_returns_hits() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(
            Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap(),
            crate::Angle::from_degrees(0.0),
            crate::Angle::from_degrees(360.0),
        )
        .unwrap();

        let intersections = circle2d_arc2d_intersections_algo(&circle, &arc);
        assert!(!intersections.is_empty());
    }

    #[test]
    fn arc_line_segment_intersection_symmetric_entry_point_returns_hits() {
        let arc = Arc2D::new(
            Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap(),
            crate::Angle::from_degrees(0.0),
            crate::Angle::from_degrees(180.0),
        )
        .unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let intersections = arc2d_line_segment2d_intersections_algo(&arc, &segment);
        assert!(!intersections.is_empty());
    }

    #[test]
    fn symmetric_intersection_wrappers_match_base_functions() {
        let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

        let tol = 1e-9;
        assert_eq!(
            circle2d_ray2d_intersections(&circle, &ray, tol),
            ray2d_circle2d_intersections(&ray, &circle, tol)
        );
        assert_eq!(
            line_segment2d_ray2d_intersection(&segment, &ray, tol),
            ray2d_line_segment2d_intersection(&ray, &segment, tol)
        );
        assert_eq!(
            circle2d_infinite_line2d_intersection(&circle, &line, tol),
            infinite_line2d_circle2d_intersection(&line, &circle, tol)
        );
        assert_eq!(
            circle2d_infinite_line2d_intersections(&circle, &line, tol),
            infinite_line2d_circle2d_intersections(&line, &circle, tol)
        );
        assert_eq!(
            line_segment2d_infinite_line2d_intersection(&segment, &line, tol),
            infinite_line2d_line_segment2d_intersection(&line, &segment, tol)
        );
        assert_eq!(
            ray2d_infinite_line2d_intersection(&ray, &line, tol),
            infinite_line2d_ray2d_intersection(&line, &ray, tol)
        );
    }

    #[test]
    fn ellipse_and_ellipse_arc_intersection_entry_points_work() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 3.0, 2.0, 0.0).unwrap();
        let ellipse_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_degrees(0.0),
            Angle::from_degrees(180.0),
        );
        let point = Point2D::new(3.0, 0.0);
        let circle = Circle2D::new(Point2D::new(2.5, 0.0), 0.75).unwrap();

        assert_eq!(
            ellipse2d_circle2d_intersection(&ellipse, &circle, 1e-6),
            circle2d_ellipse2d_intersection(&circle, &ellipse, 1e-6)
        );
        assert_eq!(
            ellipse2d_circle2d_intersections(&ellipse, &circle, 1e-6),
            circle2d_ellipse2d_intersections(&circle, &ellipse, 1e-6)
        );
        assert_eq!(
            ellipse_arc2d_point2d_intersection(&ellipse_arc, &point, 1e-6),
            Some(point)
        );
        assert_eq!(
            ellipse2d_point2d_intersection(&ellipse, &point, 1e-6),
            Some(point)
        );
    }
}
