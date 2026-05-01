//! 2D Primitive intersection algorithms
//!
//! 2D Primitive どうしの交点計算エントリポイントを提供する。
//! 共通の基礎計算は `pair_base` を利用し、このモジュールは形状ペアごとの
//! 公開 API をまとめる。

use crate::intersection::pair_base::{
    arc2d_circle2d_intersections, circle2d_circle2d_intersections,
    circle2d_line_segment2d_intersections, line_segment2d_arc2d_intersections,
    line_segment2d_circle2d_intersections, line_segment2d_line_segment2d_intersection,
};
use crate::{
    Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, IntersectionGeometry,
    IntersectionResult, IntersectionTopology, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D,
};
use geo_contracts::{
    default_parallel_cross_error_tolerance, Arc2DProperties, Circle2DProperties,
    Ellipse2DProperties, InfiniteLine2DProperties, LineSegment2DProperties, Ray2DProperties,
    Scalar, Triangle2DBoundaryAccess,
};

fn arc2d_line_segment2d_intersection_points<T: Scalar>(
    arc: &Arc2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    line_segment2d_arc2d_intersections(segment, arc, tolerance)
}

pub fn circle2d_point2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::from_tuple(circle.center());
    let distance = point.distance_to(&center);

    let opt = if (distance - circle.radius()).abs() <= tolerance {
        Some(*point)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn circle2d_circle2d_intersections_algo<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center1 = Point2D::from_tuple(circle1.center());
    let center2 = Point2D::from_tuple(circle2.center());
    let center_distance = center1.distance_to(&center2);

    if center_distance <= tolerance {
        if (circle1.radius() - circle2.radius()).abs() <= tolerance {
            return IntersectionResult::new(
                IntersectionGeometry::Coincident,
                IntersectionTopology::Coincident,
                false,
                tolerance,
            );
        }
        return IntersectionResult::disjoint(tolerance);
    }

    let points = circle2d_circle2d_intersections(circle1, circle2, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn circle2d_line_segment2d_intersections_algo<T: Scalar>(
    circle: &Circle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = circle2d_line_segment2d_intersections(circle, segment, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn arc2d_circle2d_intersections_algo<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = arc2d_circle2d_intersections(arc, circle, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn circle2d_arc2d_intersections_algo<T: Scalar>(
    circle: &Circle2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = arc2d_circle2d_intersections(arc, circle, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn line_segment2d_circle2d_intersections_algo<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = line_segment2d_circle2d_intersections(segment, circle, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn line_segment2d_arc2d_intersections_algo<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = arc2d_line_segment2d_intersection_points(arc, segment, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn arc2d_line_segment2d_intersections_algo<T: Scalar>(
    arc: &Arc2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let points = arc2d_line_segment2d_intersection_points(arc, segment, tolerance);
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn line_segment2d_line_segment2d_intersection_algo<T: Scalar>(
    segment1: &LineSegment2D<T>,
    segment2: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    if let Some(result) = collinear_segment_overlap_result(segment1, segment2, tolerance) {
        return result;
    }

    let opt = line_segment2d_line_segment2d_intersection(segment1, segment2, tolerance);
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn ray2d_line_segment2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let s1 = Point2D::new(segment.start().0, segment.start().1);
    let s2 = Point2D::new(segment.end().0, segment.end().1);

    let origin_tuple = ray.origin();
    let direction_tuple = ray.direction();
    let origin = Point2D::new(origin_tuple.0, origin_tuple.1);
    let direction = Vector2D::new(direction_tuple.0, direction_tuple.1);

    let d1 = direction;
    let d2 = Vector2D::from_points(s1, s2);

    let denominator = d1.x() * d2.y() - d1.y() * d2.x();
    if denominator.abs() <= tolerance {
        if is_point_on_line(origin, d1, s1, tolerance)
            && is_point_on_line(origin, d1, s2, tolerance)
        {
            return collinear_ray_segment_overlap_result(ray, segment, tolerance);
        }
        return IntersectionResult::from_option_point2d(None, false, tolerance);
    }

    let t1 = ((s1.x() - origin.x()) * d2.y() - (s1.y() - origin.y()) * d2.x()) / denominator;
    let t2 = ((s1.x() - origin.x()) * d1.y() - (s1.y() - origin.y()) * d1.x()) / denominator;

    let opt = if t1 >= T::ZERO - tolerance && t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance
    {
        Some(Point2D::new(
            origin.x() + t1 * d1.x(),
            origin.y() + t1 * d1.y(),
        ))
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn line_segment2d_ray2d_intersection<T: Scalar>(
    segment: &LineSegment2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    ray2d_line_segment2d_intersection(ray, segment, tolerance)
}

pub fn ray2d_circle2d_intersections<T: Scalar>(
    ray: &Ray2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
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
        return IntersectionResult::from_option_points2d(Vec::new(), false, tolerance);
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

    IntersectionResult::from_option_points2d(intersections, false, tolerance)
}

pub fn circle2d_ray2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    ray2d_circle2d_intersections(ray, circle, tolerance)
}

pub fn triangle2d_line_segment2d_intersections<T: Scalar>(
    triangle: &Triangle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
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

    IntersectionResult::from_option_points2d(intersections, false, tolerance)
}

pub fn ray2d_ellipse2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center_tuple = ellipse.center();
    let center = Point2D::new(center_tuple.0, center_tuple.1);
    let opt = if ray.distance_to_point(&center) <= ellipse.semi_major_axis() + tolerance {
        Some(center)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn arc2d_point2d_intersection<T: Scalar>(
    arc: &Arc2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::from_tuple(<Arc2D<T> as Arc2DProperties<T>>::center(arc));
    let distance = point.distance_to(&center);

    if (distance - arc.radius()).abs() > tolerance {
        return IntersectionResult::from_option_point2d(None, false, tolerance);
    }

    if !arc.contains_point_angle(*point) {
        return IntersectionResult::from_option_point2d(None, false, tolerance);
    }

    IntersectionResult::from_option_point2d(Some(*point), false, tolerance)
}

pub fn infinite_line2d_point2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let opt = if line.contains_point(point, tolerance) {
        Some(*point)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn infinite_line2d_circle2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let opt = if line.distance_to_point(&center) <= circle.radius() + tolerance {
        Some(center)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn infinite_line2d_circle2d_intersections<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let projected = line.project_point(&center);
    let dist_to_center = line.distance_to_point(&center);
    let radius = circle.radius();
    if dist_to_center > radius + tolerance {
        return IntersectionResult::from_option_points2d(Vec::new(), false, tolerance);
    }
    if (dist_to_center - radius).abs() <= tolerance {
        return IntersectionResult::from_option_points2d(vec![projected], false, tolerance);
    }
    let half_chord = (radius * radius - dist_to_center * dist_to_center).sqrt();
    let (dx, dy) = InfiniteLine2DProperties::direction(line);
    let points = vec![
        Point2D::new(
            projected.x() + half_chord * dx,
            projected.y() + half_chord * dy,
        ),
        Point2D::new(
            projected.x() - half_chord * dx,
            projected.y() - half_chord * dy,
        ),
    ];
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn circle2d_infinite_line2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line2d_circle2d_intersection(line, circle, tolerance)
}

pub fn circle2d_infinite_line2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line2d_circle2d_intersections(line, circle, tolerance)
}

pub fn infinite_line2d_line_segment2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (s1x, s1y) = LineSegment2DProperties::start(segment);
    let (s2x, s2y) = LineSegment2DProperties::end(segment);
    let (lx, ly) = InfiniteLine2DProperties::point(line);
    let (ldx, ldy) = InfiniteLine2DProperties::direction(line);
    let dx_seg = s2x - s1x;
    let dy_seg = s2y - s1y;
    let denominator = ldx * dy_seg - ldy * dx_seg;
    if denominator.abs() <= tolerance {
        let line_point = Point2D::new(lx, ly);
        let line_dir = Vector2D::new(ldx, ldy);
        let seg_start = Point2D::new(s1x, s1y);
        let seg_end = Point2D::new(s2x, s2y);
        if is_point_on_line(line_point, line_dir, seg_start, tolerance)
            && is_point_on_line(line_point, line_dir, seg_end, tolerance)
        {
            if let Some(overlap_segment) = LineSegment2D::new(seg_start, seg_end) {
                return IntersectionResult::new(
                    IntersectionGeometry::Segment2D(overlap_segment),
                    IntersectionTopology::Coincident,
                    false,
                    tolerance,
                );
            }
            return IntersectionResult::from_option_point2d(Some(seg_start), true, tolerance);
        }
        return IntersectionResult::from_option_point2d(None, false, tolerance);
    }
    let t2 = ((s1x - lx) * ldy - (s1y - ly) * ldx) / denominator;
    let opt = if t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance {
        Some(Point2D::new(s1x + t2 * dx_seg, s1y + t2 * dy_seg))
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn line_segment2d_infinite_line2d_intersection<T: Scalar>(
    segment: &LineSegment2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line2d_line_segment2d_intersection(line, segment, tolerance)
}

pub fn infinite_line2d_ray2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (ox, oy) = Ray2DProperties::origin(ray);
    let (rdx, rdy) = Ray2DProperties::direction(ray);
    let (lx, ly) = InfiniteLine2DProperties::point(line);
    let (ldx, ldy) = InfiniteLine2DProperties::direction(line);
    let denominator = ldx * rdy - ldy * rdx;
    if denominator.abs() <= tolerance {
        let line_point = Point2D::new(lx, ly);
        let line_dir = Vector2D::new(ldx, ldy);
        let ray_origin = Point2D::new(ox, oy);
        if is_point_on_line(line_point, line_dir, ray_origin, tolerance) {
            return IntersectionResult::new(
                IntersectionGeometry::Coincident,
                IntersectionTopology::Coincident,
                false,
                tolerance,
            );
        }
        return IntersectionResult::from_option_point2d(None, false, tolerance);
    }
    let t2 = ((ox - lx) * ldy - (oy - ly) * ldx) / denominator;
    let opt = if t2 >= T::ZERO - tolerance {
        Some(Point2D::new(ox + t2 * rdx, oy + t2 * rdy))
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn ray2d_infinite_line2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line2d_ray2d_intersection(line, ray, tolerance)
}

pub fn ellipse2d_point2d_intersection<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let opt = if ellipse.distance_to_point(point) <= tolerance {
        Some(*point)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn ellipse2d_circle2d_intersection<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let opt = if ellipse.distance_to_point(&center) <= circle.radius() + tolerance {
        Some(center)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn ellipse2d_circle2d_intersections<T: Scalar>(
    ellipse: &Ellipse2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = Point2D::new(circle.center().0, circle.center().1);
    let points = if ellipse.distance_to_point(&center) <= circle.radius() + tolerance {
        vec![center]
    } else {
        Vec::new()
    };
    IntersectionResult::from_option_points2d(points, false, tolerance)
}

pub fn circle2d_ellipse2d_intersection<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    ellipse2d_circle2d_intersection(ellipse, circle, tolerance)
}

pub fn circle2d_ellipse2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    ellipse2d_circle2d_intersections(ellipse, circle, tolerance)
}

pub fn ellipse_arc2d_point2d_intersection<T: Scalar>(
    arc: &EllipseArc2D<T>,
    point: &Point2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let opt = if arc.contains_point(point, tolerance) {
        Some(*point)
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

fn is_point_on_line<T: Scalar>(
    line_point: Point2D<T>,
    line_dir: Vector2D<T>,
    p: Point2D<T>,
    tolerance: T,
) -> bool {
    Vector2D::from_points(line_point, p).cross(&line_dir).abs() <= tolerance
}

fn points_near<T: Scalar>(a: Point2D<T>, b: Point2D<T>, tolerance: T) -> bool {
    (a.x() - b.x()).abs() <= tolerance && (a.y() - b.y()).abs() <= tolerance
}

fn collinear_segment_overlap_result<T: Scalar>(
    seg1: &LineSegment2D<T>,
    seg2: &LineSegment2D<T>,
    tolerance: T,
) -> Option<IntersectionResult<T>> {
    let p1 = Point2D::new(seg1.start().0, seg1.start().1);
    let p2 = Point2D::new(seg1.end().0, seg1.end().1);
    let p3 = Point2D::new(seg2.start().0, seg2.start().1);
    let p4 = Point2D::new(seg2.end().0, seg2.end().1);
    let d1 = Vector2D::from_points(p1, p2);
    let d2 = Vector2D::from_points(p3, p4);

    if d1.cross(&d2).abs() > tolerance {
        return None;
    }
    if Vector2D::from_points(p1, p3).cross(&d1).abs() > tolerance {
        return None;
    }

    let d1_norm = d1.length_squared();
    if d1_norm <= tolerance * tolerance {
        return Some(IntersectionResult::disjoint(tolerance));
    }

    let t3 = Vector2D::from_points(p1, p3).dot(&d1) / d1_norm;
    let t4 = Vector2D::from_points(p1, p4).dot(&d1) / d1_norm;
    let seg2_min = if t3 < t4 { t3 } else { t4 };
    let seg2_max = if t3 > t4 { t3 } else { t4 };

    let overlap_min = if seg2_min > T::ZERO {
        seg2_min
    } else {
        T::ZERO
    };
    let overlap_max = if seg2_max < T::ONE { seg2_max } else { T::ONE };

    if overlap_max < overlap_min - tolerance {
        return Some(IntersectionResult::disjoint(tolerance));
    }

    let overlap_start = Point2D::new(p1.x() + overlap_min * d1.x(), p1.y() + overlap_min * d1.y());
    let overlap_end = Point2D::new(p1.x() + overlap_max * d1.x(), p1.y() + overlap_max * d1.y());

    if (overlap_max - overlap_min).abs() <= tolerance {
        return Some(IntersectionResult::from_option_point2d(
            Some(overlap_start),
            true,
            tolerance,
        ));
    }

    let same_direction = points_near(p1, p3, tolerance) && points_near(p2, p4, tolerance);
    let opposite_direction = points_near(p1, p4, tolerance) && points_near(p2, p3, tolerance);
    if same_direction || opposite_direction {
        return Some(IntersectionResult::new(
            IntersectionGeometry::Coincident,
            IntersectionTopology::Coincident,
            false,
            tolerance,
        ));
    }

    let overlap_segment = LineSegment2D::new(overlap_start, overlap_end)?;
    Some(IntersectionResult::new(
        IntersectionGeometry::Segment2D(overlap_segment),
        IntersectionTopology::Coincident,
        false,
        tolerance,
    ))
}

fn collinear_ray_segment_overlap_result<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let origin = Point2D::new(ray.origin().0, ray.origin().1);
    let direction = Vector2D::new(ray.direction().0, ray.direction().1);
    let s1 = Point2D::new(segment.start().0, segment.start().1);
    let s2 = Point2D::new(segment.end().0, segment.end().1);
    let dir_norm = direction.length_squared();

    if dir_norm <= tolerance * tolerance {
        return IntersectionResult::disjoint(tolerance);
    }

    let t1 = Vector2D::from_points(origin, s1).dot(&direction) / dir_norm;
    let t2 = Vector2D::from_points(origin, s2).dot(&direction) / dir_norm;
    let seg_min = if t1 < t2 { t1 } else { t2 };
    let seg_max = if t1 > t2 { t1 } else { t2 };

    if seg_max < T::ZERO - tolerance {
        return IntersectionResult::disjoint(tolerance);
    }

    let overlap_min = if seg_min > T::ZERO { seg_min } else { T::ZERO };
    let overlap_max = seg_max;
    if overlap_max < overlap_min - tolerance {
        return IntersectionResult::disjoint(tolerance);
    }

    let overlap_start = Point2D::new(
        origin.x() + overlap_min * direction.x(),
        origin.y() + overlap_min * direction.y(),
    );

    if (overlap_max - overlap_min).abs() <= tolerance {
        return IntersectionResult::from_option_point2d(Some(overlap_start), true, tolerance);
    }

    let overlap_end = Point2D::new(
        origin.x() + overlap_max * direction.x(),
        origin.y() + overlap_max * direction.y(),
    );

    if let Some(overlap_segment) = LineSegment2D::new(overlap_start, overlap_end) {
        return IntersectionResult::new(
            IntersectionGeometry::Segment2D(overlap_segment),
            IntersectionTopology::Coincident,
            false,
            tolerance,
        );
    }

    IntersectionResult::from_option_point2d(Some(overlap_start), true, tolerance)
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

pub fn infinite_line2d_infinite_line2d_intersection<T: Scalar>(
    line1: &InfiniteLine2D<T>,
    line2: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (px, py) = InfiniteLine2DProperties::point(line1);
    let (dx1, dy1) = InfiniteLine2DProperties::direction(line1);
    let (qx, qy) = InfiniteLine2DProperties::point(line2);
    let (dx2, dy2) = InfiniteLine2DProperties::direction(line2);

    // クラメルの公式: line1.point + t1 * line1.dir = line2.point + t2 * line2.dir
    // det = dx2 * (-dy1) - dy2 * (-dx1) = -(dx2*dy1 - dy2*dx1)
    let det = dx2 * (-dy1) - dy2 * (-dx1);

    // det は無次元（方向ベクトル同士の外積）→ 無次元しきい値を使用
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if det.abs() <= par_tol {
        // 平行または同一直線
        let dp_x = px - qx;
        let dp_y = py - qy;
        // cross は長さ次元 → 距離 tolerance で判定
        let cross = dp_x * dy2 - dp_y * dx2;
        if cross.abs() <= tolerance {
            return IntersectionResult::new(
                IntersectionGeometry::Coincident,
                IntersectionTopology::Coincident,
                false,
                tolerance,
            );
        }
        return IntersectionResult::disjoint(tolerance);
    }

    let dp_x = px - qx;
    let dp_y = py - qy;
    // t1 = (dp × (-dir2)) / det
    let t1 = (dp_x * (-dy2) - dp_y * (-dx2)) / det;
    let point = Point2D::new(px + t1 * dx1, py + t1 * dy1);
    IntersectionResult::from_option_point2d(Some(point), false, tolerance)
}

pub fn infinite_line2d_infinite_line2d_intersection_sym<T: Scalar>(
    line1: &InfiniteLine2D<T>,
    line2: &InfiniteLine2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line2d_infinite_line2d_intersection(line2, line1, tolerance)
}

pub fn ray2d_ray2d_intersection<T: Scalar>(
    ray1: &Ray2D<T>,
    ray2: &Ray2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (ox1, oy1) = Ray2DProperties::origin(ray1);
    let (dx1, dy1) = Ray2DProperties::direction(ray1);
    let (ox2, oy2) = Ray2DProperties::origin(ray2);
    let (dx2, dy2) = Ray2DProperties::direction(ray2);

    // ray1: P = (ox1, oy1) + t1 * (dx1, dy1),  t1 >= 0
    // ray2: Q = (ox2, oy2) + t2 * (dx2, dy2),  t2 >= 0
    let denominator = dx1 * dy2 - dy1 * dx2;

    // denominator は無次元（方向ベクトル同士の外積）→ 無次元しきい値を使用
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if denominator.abs() <= par_tol {
        // 平行または同一方向
        let dp_x = ox2 - ox1;
        let dp_y = oy2 - oy1;
        // cross は長さ次元 → 距離 tolerance で判定
        let cross = dp_x * dy1 - dp_y * dx1;
        if cross.abs() <= tolerance {
            // コリニア: 方向・重複区間を確認
            let dir_sq = dx1 * dx1 + dy1 * dy1;
            if dir_sq <= par_tol {
                return IntersectionResult::disjoint(tolerance);
            }
            // ray1 パラメータ系での ray2 起点位置
            let t_base = (dp_x * dx1 + dp_y * dy1) / dir_sq;
            // 方向の一致・逆方向を確認
            let q = dx1 * dx2 + dy1 * dy2;
            if q >= T::ZERO {
                // 同方向: 無限重複 → Coincident
                return IntersectionResult::new(
                    IntersectionGeometry::Coincident,
                    IntersectionTopology::Coincident,
                    true,
                    tolerance,
                );
            }
            // 逆方向: 有効重複区間 [0, t_base]
            if t_base < T::ZERO - tolerance {
                return IntersectionResult::disjoint(tolerance);
            }
            if t_base <= tolerance {
                // 起点共有のみ
                let p = Point2D::new(ox1, oy1);
                return IntersectionResult::from_option_point2d(Some(p), true, tolerance);
            }
            // 有限セグメント重複: O1 から O2 へ
            let p_start = Point2D::new(ox1, oy1);
            let p_end = Point2D::new(ox2, oy2);
            if let Some(seg) = LineSegment2D::new(p_start, p_end) {
                return IntersectionResult::new(
                    IntersectionGeometry::Segment2D(seg),
                    IntersectionTopology::Coincident,
                    true,
                    tolerance,
                );
            }
            return IntersectionResult::from_option_point2d(Some(p_start), true, tolerance);
        }
        return IntersectionResult::disjoint(tolerance);
    }

    let dp_x = ox2 - ox1;
    let dp_y = oy2 - oy1;
    let t1 = (dp_x * dy2 - dp_y * dx2) / denominator;
    let t2 = (dp_x * dy1 - dp_y * dx1) / denominator;

    if t1 >= T::ZERO - tolerance && t2 >= T::ZERO - tolerance {
        let point = Point2D::new(ox1 + t1 * dx1, oy1 + t1 * dy1);
        IntersectionResult::from_option_point2d(Some(point), false, tolerance)
    } else {
        IntersectionResult::disjoint(tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        arc2d_line_segment2d_intersections_algo, arc2d_point2d_intersection,
        circle2d_arc2d_intersections_algo, circle2d_circle2d_intersections_algo,
        circle2d_ellipse2d_intersection, circle2d_ellipse2d_intersections,
        circle2d_infinite_line2d_intersection, circle2d_infinite_line2d_intersections,
        circle2d_point2d_intersection, circle2d_ray2d_intersections,
        ellipse2d_circle2d_intersection, ellipse2d_circle2d_intersections,
        ellipse2d_point2d_intersection, ellipse_arc2d_point2d_intersection,
        infinite_line2d_circle2d_intersection, infinite_line2d_circle2d_intersections,
        infinite_line2d_infinite_line2d_intersection, infinite_line2d_line_segment2d_intersection,
        infinite_line2d_point2d_intersection, infinite_line2d_ray2d_intersection,
        line_segment2d_infinite_line2d_intersection,
        line_segment2d_line_segment2d_intersection_algo, line_segment2d_ray2d_intersection,
        ray2d_circle2d_intersections, ray2d_infinite_line2d_intersection,
        ray2d_line_segment2d_intersection, ray2d_ray2d_intersection,
    };
    use crate::{
        Angle, Arc2D, Circle2D, Ellipse2D, EllipseArc2D, InfiniteLine2D, IntersectionGeometry,
        IntersectionTopology, LineSegment2D, Point2D, Ray2D, Vector2D,
    };
    use analysis::test_constants;
    use geo_contracts::LineSegment2DProperties;

    const STANDARD_TEST_TOLERANCE_F64: f64 = test_constants::DISTANCE_TOLERANCE_F64;
    const ELLIPSE_ENTRY_TEST_TOLERANCE_F64: f64 = 1.0e-6;

    // --- infinite_line2d_infinite_line2d_intersection ---

    #[test]
    fn infinite_line2d_intersection_non_parallel_returns_point() {
        let line1 = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let line2 = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(0.0, 1.0)).unwrap();
        let result = infinite_line2d_infinite_line2d_intersection(
            &line1,
            &line2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(result.intersects(), "non-parallel lines must intersect");
        assert_eq!(result.topology, IntersectionTopology::Crossing);
        if let IntersectionGeometry::Point2D(p) = result.geometry {
            assert!(p.x().abs() < STANDARD_TEST_TOLERANCE_F64);
            assert!(p.y().abs() < STANDARD_TEST_TOLERANCE_F64);
        } else {
            panic!("expected Point2D geometry");
        }
    }

    #[test]
    fn infinite_line2d_intersection_parallel_is_disjoint() {
        let line1 = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let line2 = InfiniteLine2D::new(Point2D::new(0.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();
        let result = infinite_line2d_infinite_line2d_intersection(
            &line1,
            &line2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(!result.intersects(), "parallel lines must be disjoint");
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn infinite_line2d_intersection_coincident_lines() {
        let line1 = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let line2 = InfiniteLine2D::new(Point2D::new(3.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let result = infinite_line2d_infinite_line2d_intersection(
            &line1,
            &line2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(result.intersects(), "coincident lines must intersect");
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        assert!(
            matches!(result.geometry, IntersectionGeometry::Coincident),
            "expected Coincident geometry"
        );
    }

    // --- ray2d_ray2d_intersection ---

    #[test]
    fn ray2d_ray2d_intersection_non_parallel_both_valid() {
        // 両 t >= 0 で交差
        let ray1 = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(1.0, -1.0), Vector2D::new(0.0, 1.0)).unwrap();
        let result = ray2d_ray2d_intersection(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(
            result.intersects(),
            "rays crossing in valid range must intersect"
        );
    }

    #[test]
    fn ray2d_ray2d_intersection_non_parallel_out_of_range_is_disjoint() {
        // 交点が一方のRayの後ろ側
        let ray1 = Ray2D::new(Point2D::new(2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(0.0, 1.0)).unwrap();
        let result = ray2d_ray2d_intersection(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(
            !result.intersects(),
            "intersection point is behind ray1: must be disjoint"
        );
    }

    #[test]
    fn ray2d_ray2d_intersection_collinear_opposite_direction_returns_segment() {
        // 逆向きコリニア: ray1=(0,0)+t*(1,0), ray2=(2,0)+t*(-1,0) → [0,2] の Segment2D
        let ray1 = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(2.0, 0.0), Vector2D::new(-1.0, 0.0)).unwrap();
        let result = ray2d_ray2d_intersection(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(
            result.intersects(),
            "collinear opposite rays with overlap must intersect"
        );
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        assert!(
            matches!(result.geometry, IntersectionGeometry::Segment2D(_)),
            "expected Segment2D for finite overlap, got {:?}",
            result.geometry
        );
    }

    #[test]
    fn ray2d_ray2d_intersection_collinear_pointing_away_is_disjoint() {
        // 逆向きコリニアで反対方向を向いている: 重複なし
        let ray1 = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(-1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let result = ray2d_ray2d_intersection(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(
            !result.intersects(),
            "collinear rays pointing away from each other must be disjoint"
        );
    }

    #[test]
    fn circle_point_intersection_on_boundary_is_crossing() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let point = Point2D::new(1.0, 0.0);

        let result = circle2d_point2d_intersection(&circle, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    #[test]
    fn circle_point_intersection_outside_is_disjoint() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let point = Point2D::new(2.0, 0.0);

        let result = circle2d_point2d_intersection(&circle, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn arc_point_intersection_on_arc_is_crossing() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(circle, Angle::from_degrees(0.0), Angle::from_degrees(180.0)).unwrap();
        let point = Point2D::new(0.0, 1.0);

        let result = arc2d_point2d_intersection(&arc, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    #[test]
    fn arc_point_intersection_outside_arc_range_is_disjoint() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(circle, Angle::from_degrees(0.0), Angle::from_degrees(90.0)).unwrap();
        let point = Point2D::new(-1.0, 0.0);

        let result = arc2d_point2d_intersection(&arc, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn infinite_line_point_intersection_on_line_is_crossing() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let point = Point2D::new(5.0, 0.0);

        let result =
            infinite_line2d_point2d_intersection(&line, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    #[test]
    fn infinite_line_point_intersection_off_line_is_disjoint() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let point = Point2D::new(0.0, 1.0);

        let result =
            infinite_line2d_point2d_intersection(&line, &point, STANDARD_TEST_TOLERANCE_F64);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn circle_circle_intersection_returns_two_points() {
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap();

        let result =
            circle2d_circle2d_intersections_algo(&circle1, &circle2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
        match &result.geometry {
            IntersectionGeometry::Points2D(points) => assert_eq!(points.len(), 2),
            _ => panic!("expected Points2D geometry"),
        }
    }

    #[test]
    fn line_segment_line_segment_intersection_returns_crossing_point() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let segment2 = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

        let result = line_segment2d_line_segment2d_intersection_algo(
            &segment1,
            &segment2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    #[test]
    fn circle_circle_identical_returns_coincident() {
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let result =
            circle2d_circle2d_intersections_algo(&circle1, &circle2, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        match result.geometry {
            IntersectionGeometry::Coincident => {}
            _ => panic!("expected Coincident geometry"),
        }
    }

    #[test]
    fn circle_circle_concentric_different_radius_is_disjoint() {
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

        let result =
            circle2d_circle2d_intersections_algo(&circle1, &circle2, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn line_segment_line_segment_collinear_overlap_returns_segment2d() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 0.0)).unwrap();
        let segment2 = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        let result = line_segment2d_line_segment2d_intersection_algo(
            &segment1,
            &segment2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        match result.geometry {
            IntersectionGeometry::Segment2D(seg) => {
                let start = seg.start();
                let end = seg.end();
                assert!((start.0 - 1.0).abs() < STANDARD_TEST_TOLERANCE_F64);
                assert!((end.0 - 3.0).abs() < STANDARD_TEST_TOLERANCE_F64);
            }
            _ => panic!("expected Segment2D geometry"),
        }
    }

    #[test]
    fn line_segment_line_segment_collinear_touching_endpoint_returns_touching() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();
        let segment2 = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let result = line_segment2d_line_segment2d_intersection_algo(
            &segment1,
            &segment2,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(result.topology, IntersectionTopology::Touching);
        match result.geometry {
            IntersectionGeometry::Point2D(p) => {
                assert!((p.x() - 1.0).abs() < STANDARD_TEST_TOLERANCE_F64);
            }
            _ => panic!("expected Point2D geometry"),
        }
    }

    #[test]
    fn ray_circle_intersection_returns_forward_hits_only() {
        let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let result = ray2d_circle2d_intersections(&ray, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
        match &result.geometry {
            IntersectionGeometry::Points2D(points) => {
                assert_eq!(points.len(), 2);
                assert!(points.iter().all(|point| point.x() >= -1.0));
            }
            _ => panic!("expected Points2D geometry"),
        }
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

        let result = circle2d_arc2d_intersections_algo(&circle, &arc, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
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

        let result =
            arc2d_line_segment2d_intersections_algo(&arc, &segment, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.intersects());
    }

    #[test]
    fn symmetric_intersection_wrappers_match_base_functions() {
        let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

        let tol = STANDARD_TEST_TOLERANCE_F64;
        let circle_ray = circle2d_ray2d_intersections(&circle, &ray, tol);
        let ray_circle = ray2d_circle2d_intersections(&ray, &circle, tol);
        assert_eq!(circle_ray.topology, ray_circle.topology);
        assert_eq!(circle_ray.is_tangent, ray_circle.is_tangent);
        let seg_ray = line_segment2d_ray2d_intersection(&segment, &ray, tol);
        let ray_seg = ray2d_line_segment2d_intersection(&ray, &segment, tol);
        assert_eq!(seg_ray.topology, ray_seg.topology);
        assert_eq!(seg_ray.is_tangent, ray_seg.is_tangent);
        let circ_line = circle2d_infinite_line2d_intersection(&circle, &line, tol);
        let line_circ = infinite_line2d_circle2d_intersection(&line, &circle, tol);
        assert_eq!(circ_line.topology, line_circ.topology);
        assert_eq!(circ_line.is_tangent, line_circ.is_tangent);
        let circle_line_points = circle2d_infinite_line2d_intersections(&circle, &line, tol);
        let line_circle_points = infinite_line2d_circle2d_intersections(&line, &circle, tol);
        assert_eq!(circle_line_points.topology, line_circle_points.topology);
        assert_eq!(circle_line_points.is_tangent, line_circle_points.is_tangent);
        let seg_line = line_segment2d_infinite_line2d_intersection(&segment, &line, tol);
        let line_seg = infinite_line2d_line_segment2d_intersection(&line, &segment, tol);
        assert_eq!(seg_line.topology, line_seg.topology);
        assert_eq!(seg_line.is_tangent, line_seg.is_tangent);
        let ray_line = ray2d_infinite_line2d_intersection(&ray, &line, tol);
        let line_ray = infinite_line2d_ray2d_intersection(&line, &ray, tol);
        assert_eq!(ray_line.topology, line_ray.topology);
        assert_eq!(ray_line.is_tangent, line_ray.is_tangent);
    }

    #[test]
    fn ray_segment_collinear_overlap_returns_segment2d() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let segment = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(3.0, 0.0)).unwrap();

        let result = ray2d_line_segment2d_intersection(&ray, &segment, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        match result.geometry {
            IntersectionGeometry::Segment2D(_) => {}
            _ => panic!("expected Segment2D geometry"),
        }
    }

    #[test]
    fn infinite_line_segment_collinear_returns_segment2d() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let result = infinite_line2d_line_segment2d_intersection(
            &line,
            &segment,
            STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        match result.geometry {
            IntersectionGeometry::Segment2D(_) => {}
            _ => panic!("expected Segment2D geometry"),
        }
    }

    #[test]
    fn infinite_line_ray_collinear_returns_coincident() {
        let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray = Ray2D::new(Point2D::new(1.0, 0.0), Vector2D::new(2.0, 0.0)).unwrap();

        let result = infinite_line2d_ray2d_intersection(&line, &ray, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(result.topology, IntersectionTopology::Coincident);
        match result.geometry {
            IntersectionGeometry::Coincident => {}
            _ => panic!("expected Coincident geometry"),
        }
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

        let ellipse_circle =
            ellipse2d_circle2d_intersection(&ellipse, &circle, ELLIPSE_ENTRY_TEST_TOLERANCE_F64);
        let circle_ellipse =
            circle2d_ellipse2d_intersection(&circle, &ellipse, ELLIPSE_ENTRY_TEST_TOLERANCE_F64);
        assert_eq!(ellipse_circle.topology, circle_ellipse.topology);
        assert_eq!(ellipse_circle.is_tangent, circle_ellipse.is_tangent);
        let ellipse_circle_points =
            ellipse2d_circle2d_intersections(&ellipse, &circle, ELLIPSE_ENTRY_TEST_TOLERANCE_F64);
        let circle_ellipse_points =
            circle2d_ellipse2d_intersections(&circle, &ellipse, ELLIPSE_ENTRY_TEST_TOLERANCE_F64);
        assert_eq!(
            ellipse_circle_points.topology,
            circle_ellipse_points.topology
        );
        assert_eq!(
            ellipse_circle_points.is_tangent,
            circle_ellipse_points.is_tangent
        );
        assert!(ellipse_arc2d_point2d_intersection(
            &ellipse_arc,
            &point,
            ELLIPSE_ENTRY_TEST_TOLERANCE_F64
        )
        .intersects());
        assert!(
            ellipse2d_point2d_intersection(&ellipse, &point, ELLIPSE_ENTRY_TEST_TOLERANCE_F64)
                .intersects()
        );
    }
}
