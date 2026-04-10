use super::shared::{
    point_intersection_if, point_matches_either_segment_endpoint, triangle3d_vertex_points,
};
use crate::{
    Arc3D, Circle3D, Ellipse3D, InfiniteLine3D, IntersectionResult, LineSegment3D, Plane3D,
    Point3D, Ray3D, Triangle3D, Vector3D,
};
use geo_contracts::{
    Arc3DEndpoint, Arc3DProperties, Circle3DProperties, InfiniteLine3DProperties, Scalar,
};

fn arc3d_point3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::arc3d_point3d_distance(arc, point) <= tolerance
            && arc.contains_point_angle(Point3D::new(point.x(), point.y(), point.z())),
    )
}

pub fn arc3d_point3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_point3d_intersection_raw(arc, point, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_line_segment3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (sx, sy, sz) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc);
    let (ex, ey, ez) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc);
    let arc_start = Point3D::new(sx, sy, sz);
    let arc_end = Point3D::new(ex, ey, ez);
    let d_seg_s = crate::distance::arc3d_point3d_distance(arc, &segment.start());
    let d_seg_e = crate::distance::arc3d_point3d_distance(arc, &segment.end());
    if d_seg_s <= tolerance {
        return Some(segment.start());
    }
    if d_seg_e <= tolerance {
        return Some(segment.end());
    }
    if point_matches_either_segment_endpoint(arc_start, segment, tolerance) {
        Some(arc_start)
    } else if point_matches_either_segment_endpoint(arc_end, segment, tolerance) {
        Some(arc_end)
    } else {
        None
    }
}

pub fn arc3d_line_segment3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_line_segment3d_intersection_raw(arc, segment, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_ray3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let d = crate::distance::arc3d_point3d_distance(arc, &origin);
    if d <= tolerance {
        Some(origin)
    } else {
        None
    }
}

pub fn arc3d_ray3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_ray3d_intersection_raw(arc, ray, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_infinite_line3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let pt = Point3D::new(px, py, pz);
    let d = crate::distance::arc3d_point3d_distance(arc, &pt);
    if d <= tolerance {
        Some(pt)
    } else {
        None
    }
}

pub fn arc3d_infinite_line3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_infinite_line3d_intersection_raw(arc, line, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_arc3d_intersection_raw<T: Scalar>(
    arc_a: &Arc3D<T>,
    arc_b: &Arc3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (s1x, s1y, s1z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_a);
    let (e1x, e1y, e1z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_a);
    let (s2x, s2y, s2z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_b);
    let (e2x, e2y, e2z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_b);
    let pa = Point3D::new(s1x, s1y, s1z);
    let pb = Point3D::new(e1x, e1y, e1z);
    let pc = Point3D::new(s2x, s2y, s2z);
    let pd = Point3D::new(e2x, e2y, e2z);
    if Vector3D::from_points(&pa, &pc).magnitude() <= tolerance {
        return Some(pa);
    }
    if Vector3D::from_points(&pa, &pd).magnitude() <= tolerance {
        return Some(pa);
    }
    if Vector3D::from_points(&pb, &pc).magnitude() <= tolerance {
        return Some(pb);
    }
    if Vector3D::from_points(&pb, &pd).magnitude() <= tolerance {
        return Some(pb);
    }
    None
}

pub fn arc3d_arc3d_intersection<T: Scalar>(
    arc_a: &Arc3D<T>,
    arc_b: &Arc3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_arc3d_intersection_raw(arc_a, arc_b, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_point3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::circle3d_point3d_distance(circle, point) <= tolerance,
    )
}

pub fn circle3d_point3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_point3d_intersection_raw(circle, point, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_line_segment3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    if crate::distance::circle3d_point3d_distance(circle, &start) <= tolerance {
        return Some(start);
    }
    let end = segment.end();
    if crate::distance::circle3d_point3d_distance(circle, &end) <= tolerance {
        return Some(end);
    }
    None
}

pub fn circle3d_line_segment3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_line_segment3d_intersection_raw(circle, segment, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_ray3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    if crate::distance::circle3d_point3d_distance(circle, &origin) <= tolerance {
        Some(origin)
    } else {
        None
    }
}

pub fn circle3d_ray3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_ray3d_intersection_raw(circle, ray, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_infinite_line3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let pt = Point3D::new(px, py, pz);
    if crate::distance::circle3d_point3d_distance(circle, &pt) <= tolerance {
        Some(pt)
    } else {
        None
    }
}

pub fn circle3d_infinite_line3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_infinite_line3d_intersection_raw(circle, line, tolerance),
        false,
        tolerance,
    )
}

pub fn circle3d_circle3d_intersection<T: Scalar>(
    _circle_a: &Circle3D<T>,
    _circle_b: &Circle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(None, false, tolerance)
}

fn ellipse3d_point3d_intersection_raw<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::ellipse3d_point3d_distance(ellipse, point) <= tolerance,
    )
}

pub fn ellipse3d_point3d_intersection<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipse3d_point3d_intersection_raw(ellipse, point, tolerance),
        false,
        tolerance,
    )
}

pub fn ellipse3d_circle3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (cx, cy, cz) = Circle3DProperties::center(circle);
    let center = Point3D::new(cx, cy, cz);
    let dist = crate::distance::ellipse3d_point3d_distance(ellipse, &center);
    let points = if dist <= Circle3DProperties::radius(circle) + tolerance {
        vec![center]
    } else {
        Vec::new()
    };
    IntersectionResult::from_option_points(points, false, tolerance)
}

pub fn ellipse3d_arc3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (cx, cy, cz) = Arc3DProperties::center(arc);
    let center = Point3D::new(cx, cy, cz);
    let dist = crate::distance::ellipse3d_point3d_distance(ellipse, &center);
    let points = if dist <= Arc3DProperties::radius(arc) + tolerance {
        vec![center]
    } else {
        Vec::new()
    };
    IntersectionResult::from_option_points(points, false, tolerance)
}

pub fn ellipse3d_line_segment3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let start = segment.start();
    let end = segment.end();
    let mut intersections = Vec::new();
    if crate::distance::ellipse3d_point3d_distance(ellipse, &start) <= tolerance {
        intersections.push(start);
    }
    if crate::distance::ellipse3d_point3d_distance(ellipse, &end) <= tolerance {
        intersections.push(end);
    }
    IntersectionResult::from_option_points(intersections, false, tolerance)
}

pub fn ellipse3d_infinite_line3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let point = Point3D::new(px, py, pz);
    let dist = crate::distance::ellipse3d_point3d_distance(ellipse, &point);
    let points = if dist <= tolerance {
        vec![point]
    } else {
        Vec::new()
    };
    IntersectionResult::from_option_points(points, false, tolerance)
}

pub fn ellipse3d_ray3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let o = ray.origin();
    let dist = crate::distance::ellipse3d_point3d_distance(ellipse, &o);
    let points = if dist <= tolerance {
        vec![o]
    } else {
        Vec::new()
    };
    IntersectionResult::from_option_points(points, false, tolerance)
}

fn ellipse3d_plane3d_intersection_raw<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let center = ellipse.center();
    if plane.distance_to_point(center).abs() <= tolerance {
        Some(ellipse.center())
    } else {
        None
    }
}

pub fn ellipse3d_plane3d_intersection<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipse3d_plane3d_intersection_raw(ellipse, plane, tolerance),
        false,
        tolerance,
    )
}

pub fn ellipse3d_triangle3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    for point in triangle3d_vertex_points(triangle) {
        if crate::distance::ellipse3d_point3d_distance(ellipse, &point) <= tolerance {
            intersections.push(point);
        }
    }
    IntersectionResult::from_option_points(intersections, false, tolerance)
}

pub fn ellipse3d_ellipse3d_intersections<T: Scalar>(
    _ellipse_a: &Ellipse3D<T>,
    _ellipse_b: &Ellipse3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_points(Vec::new(), false, tolerance)
}
