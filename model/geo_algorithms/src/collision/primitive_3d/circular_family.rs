use super::shared::triangle3d_vertex_points;
use crate::{
    Arc3D, Circle3D, Ellipse3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D,
    Vector3D,
};
use geo_contracts::{
    Arc3DEndpoint, Arc3DProperties, Circle3DProperties, InfiniteLine3DProperties, Scalar,
};

pub fn ellipse3d_point3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::ellipse3d_point3d_distance(ellipse, point) <= tolerance
}

pub fn ellipse3d_circle3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &center) <= circle.radius() + tolerance
}

pub fn ellipse3d_arc3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = Arc3DProperties::center(arc);
    let arc_radius = Arc3DProperties::radius(arc);
    let center = Point3D::new(cx, cy, cz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &center) <= arc_radius + tolerance
}

pub fn ellipse3d_line_segment3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    let dist_start = crate::distance::ellipse3d_point3d_distance(ellipse, &s);
    let dist_end = crate::distance::ellipse3d_point3d_distance(ellipse, &e);
    let two = T::from_f64(2.0);
    let mid = Point3D::new(
        (s.x() + e.x()) / two,
        (s.y() + e.y()) / two,
        (s.z() + e.z()) / two,
    );
    let dist_mid = crate::distance::ellipse3d_point3d_distance(ellipse, &mid);
    dist_start <= tolerance || dist_end <= tolerance || dist_mid <= tolerance
}

pub fn ellipse3d_infinite_line3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &point) <= tolerance
}

pub fn ellipse3d_ray3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::ellipse3d_point3d_distance(ellipse, &o) <= tolerance
}

pub fn ellipse3d_plane3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let center = ellipse.center();
    plane.distance_to_point(center).abs() <= tolerance
}

pub fn ellipse3d_triangle3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let [point_a, point_b, point_c] = triangle3d_vertex_points(triangle);
    let dist_a = crate::distance::ellipse3d_point3d_distance(ellipse, &point_a);
    let dist_b = crate::distance::ellipse3d_point3d_distance(ellipse, &point_b);
    let dist_c = crate::distance::ellipse3d_point3d_distance(ellipse, &point_c);
    let three = T::from_f64(3.0);
    let centroid = Point3D::new(
        (point_a.x() + point_b.x() + point_c.x()) / three,
        (point_a.y() + point_b.y() + point_c.y()) / three,
        (point_a.z() + point_b.z() + point_c.z()) / three,
    );
    let dist_centroid = crate::distance::ellipse3d_point3d_distance(ellipse, &centroid);
    dist_a <= tolerance || dist_b <= tolerance || dist_c <= tolerance || dist_centroid <= tolerance
}

pub fn ellipse3d_ellipse3d_collides<T: Scalar>(
    ellipse_a: &Ellipse3D<T>,
    ellipse_b: &Ellipse3D<T>,
    tolerance: T,
) -> bool {
    let c1 = ellipse_a.center();
    let c2 = ellipse_b.center();
    let center_dist = c1.distance_to(&c2);
    let sum_semi_major = ellipse_a.semi_major_axis() + ellipse_b.semi_major_axis();
    center_dist <= sum_semi_major + tolerance
}

pub fn arc3d_point3d_collides<T: Scalar>(arc: &Arc3D<T>, point: &Point3D<T>, tolerance: T) -> bool {
    crate::distance::arc3d_point3d_distance(arc, point) <= tolerance
        && arc.contains_point_angle(Point3D::new(point.x(), point.y(), point.z()))
}

pub fn arc3d_line_segment3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let (sx, sy, sz) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc);
    let (ex, ey, ez) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc);
    let arc_start = Point3D::new(sx, sy, sz);
    let arc_end = Point3D::new(ex, ey, ez);
    crate::distance::arc3d_point3d_distance(arc, &segment.start()) <= tolerance
        || crate::distance::arc3d_point3d_distance(arc, &segment.end()) <= tolerance
        || {
            let d1 = Vector3D::from_points(&arc_start, &segment.start()).magnitude();
            let d2 = Vector3D::from_points(&arc_end, &segment.start()).magnitude();
            d1 <= tolerance || d2 <= tolerance
        }
}

pub fn arc3d_ray3d_collides<T: Scalar>(arc: &Arc3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    crate::distance::arc3d_point3d_distance(arc, &ray.origin()) <= tolerance
}

pub fn arc3d_infinite_line3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    crate::distance::arc3d_point3d_distance(arc, &Point3D::new(px, py, pz)) <= tolerance
}

pub fn arc3d_arc3d_collides<T: Scalar>(arc_a: &Arc3D<T>, arc_b: &Arc3D<T>, tolerance: T) -> bool {
    let (s1x, s1y, s1z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_a);
    let (e1x, e1y, e1z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_a);
    let (s2x, s2y, s2z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_b);
    let (e2x, e2y, e2z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_b);
    let pa = Point3D::new(s1x, s1y, s1z);
    let pb = Point3D::new(e1x, e1y, e1z);
    let pc = Point3D::new(s2x, s2y, s2z);
    let pd = Point3D::new(e2x, e2y, e2z);
    Vector3D::from_points(&pa, &pc).magnitude() <= tolerance
        || Vector3D::from_points(&pa, &pd).magnitude() <= tolerance
        || Vector3D::from_points(&pb, &pc).magnitude() <= tolerance
        || Vector3D::from_points(&pb, &pd).magnitude() <= tolerance
}

pub fn circle3d_point3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::circle3d_point3d_distance(circle, point) <= tolerance
}

pub fn circle3d_line_segment3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let dist_start = crate::distance::circle3d_point3d_distance(circle, &segment.start());
    let dist_end = crate::distance::circle3d_point3d_distance(circle, &segment.end());
    let mid_x = (segment.start().x() + segment.end().x()) / T::from_f64(2.0);
    let mid_y = (segment.start().y() + segment.end().y()) / T::from_f64(2.0);
    let mid_z = (segment.start().z() + segment.end().z()) / T::from_f64(2.0);
    let midpoint = Point3D::new(mid_x, mid_y, mid_z);
    dist_start <= tolerance
        || dist_end <= tolerance
        || crate::distance::circle3d_point3d_distance(circle, &midpoint) <= tolerance
}

pub fn circle3d_ray3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::circle3d_point3d_distance(circle, &ray.origin()) <= tolerance
}

pub fn circle3d_infinite_line3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point_on_line = Point3D::new(px, py, pz);
    crate::distance::circle3d_point3d_distance(circle, &point_on_line) <= tolerance
}

pub fn circle3d_circle3d_collides<T: Scalar>(
    circle_a: &Circle3D<T>,
    circle_b: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = circle_a.center();
    let (bx, by, bz) = circle_b.center();
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    dist <= circle_a.radius() + circle_b.radius() + tolerance
}
