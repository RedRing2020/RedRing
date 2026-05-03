//! 交点計算のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の幾何計算を集約する。

use super::primitive_3d::line_line_intersection_raw;
use crate::{
    Arc2D, Circle2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Plane3D, Point2D, Point3D,
    Ray3D, SphericalSurface3D, Triangle3D, Vector3D,
};
use geo_contracts::{
    Arc2DProperties, Circle2DProperties, InfiniteLine3DProperties, LineSegment2DProperties, Scalar,
    SphericalSurface3DProperties, Triangle3DBoundaryAccess,
};

#[cfg(test)]
const STANDARD_TEST_TOLERANCE_F64: f64 = analysis::test_constants::DISTANCE_TOLERANCE_F64;
#[cfg(test)]
const SMALL_GAP_WITHIN_TOLERANCE_F64: f64 = STANDARD_TEST_TOLERANCE_F64 / 10.0;

pub fn circle2d_circle2d_intersections<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center1 = circle1.center();
    let center2 = circle2.center();
    let center1_point = Point2D::from_tuple(center1);
    let center2_point = Point2D::from_tuple(center2);
    let r1 = circle1.radius();
    let r2 = circle2.radius();

    let dx = center2.0 - center1.0;
    let dy = center2.1 - center1.1;
    let d = center1_point.distance_to(&center2_point);

    if d > r1 + r2 + tolerance || d + tolerance < (r1 - r2).abs() || d.abs() <= tolerance {
        return result;
    }

    let a = (r1 * r1 - r2 * r2 + d * d) / ((T::ONE + T::ONE) * d);
    let h_squared = r1 * r1 - a * a;
    if h_squared < -tolerance {
        return result;
    }

    let h_squared = if h_squared.abs() <= tolerance {
        T::ZERO
    } else {
        h_squared
    };

    let h = h_squared.sqrt();
    let px = center1.0 + a * dx / d;
    let py = center1.1 + a * dy / d;

    if h.abs() <= tolerance {
        result.push(Point2D::new(px, py));
    } else {
        result.push(Point2D::new(px + h * dy / d, py - h * dx / d));
        result.push(Point2D::new(px - h * dy / d, py + h * dx / d));
    }

    result
}

pub fn circle2d_line_segment2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    line_segment2d_circle2d_intersections(segment, circle, tolerance)
}

pub fn arc2d_circle2d_intersections<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let base_circle = Circle2D::new(
        Point2D::new(center_x, center_y),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        circle2d_circle2d_intersections(&base_circle, circle, tolerance)
            .into_iter()
            .filter(|p| arc.contains_point_angle(*p))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn line_segment2d_circle2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start();
    let end = segment.end();

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let fx = start.0 - center.0;
    let fy = start.1 - center.1;

    let start_point = Point2D::from_tuple(start);
    let end_point = Point2D::from_tuple(end);
    let a = start_point.distance_squared_to(&end_point);
    if a.abs() <= tolerance {
        return result;
    }

    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < -tolerance {
        return result;
    }

    let discriminant = if discriminant.abs() <= tolerance {
        T::ZERO
    } else {
        discriminant
    };

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= -tolerance && t1 <= T::ONE + tolerance {
        let x = start.0 + t1 * dx;
        let y = start.1 + t1 * dy;
        result.push(Point2D::new(x, y));
    }

    if t2 >= -tolerance && t2 <= T::ONE + tolerance && (t2 - t1).abs() > tolerance {
        let x = start.0 + t2 * dx;
        let y = start.1 + t2 * dy;
        result.push(Point2D::new(x, y));
    }

    result
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(
        Point2D::new(arc.center().0, arc.center().1),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle, tolerance)
            .into_iter()
            .filter(|p| arc.contains_point_angle(*p))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn line_segment2d_line_segment2d_intersection<T: Scalar>(
    seg1: &LineSegment2D<T>,
    seg2: &LineSegment2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let p1 = seg1.start();
    let p2 = seg1.end();
    let p3 = seg2.start();
    let p4 = seg2.end();

    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denominator = d1x * d2y - d1y * d2x;
    if denominator.abs() <= tolerance {
        return None;
    }

    let t1 = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denominator;
    let t2 = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denominator;

    if t1 >= -tolerance && t1 <= T::ONE + tolerance && t2 >= -tolerance && t2 <= T::ONE + tolerance
    {
        let x = p1.0 + t1 * d1x;
        let y = p1.1 + t1 * d1y;
        Some(Point2D::new(x, y))
    } else {
        None
    }
}

pub fn line_segment3d_spherical_surface3d_intersections<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    let mut result = Vec::new();

    let center_tuple = sphere.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = sphere.radius();
    let start = segment.start();
    let end = segment.end();

    let d = Vector3D::from_points(&start, &end);
    let f = Vector3D::from_points(&center, &start);

    let a = d.dot(&d);
    if a.abs() <= T::EPSILON {
        return result;
    }

    let b = (f.dot(&d)) * (T::ONE + T::ONE);
    let c = f.dot(&f) - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= T::ZERO && t1 <= T::ONE {
        let point = Point3D::new(
            start.x() + t1 * d.x(),
            start.y() + t1 * d.y(),
            start.z() + t1 * d.z(),
        );
        result.push(point);
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > T::EPSILON {
        let point = Point3D::new(
            start.x() + t2 * d.x(),
            start.y() + t2 * d.y(),
            start.z() + t2 * d.z(),
        );
        result.push(point);
    }

    result
}

pub fn line_segment3d_line_segment3d_intersection<T: Scalar>(
    seg1: &LineSegment3D<T>,
    seg2: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let p1 = seg1.start();
    let p2 = seg1.end();
    let p3 = seg2.start();
    let p4 = seg2.end();

    let d1 = Vector3D::from_points(&p1, &p2);
    let d2 = Vector3D::from_points(&p3, &p4);
    let r = Vector3D::from_points(&p3, &p1);

    let a = d1.dot(&d1);
    let b = d1.dot(&d2);
    let c = d2.dot(&d2);
    let d = d1.dot(&r);
    let e = d2.dot(&r);

    let denom = a * c - b * b;
    if denom.abs() < T::EPSILON {
        return None;
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    if s >= T::ZERO && s <= T::ONE && t >= T::ZERO && t <= T::ONE {
        let point1 = Point3D::new(
            p1.x() + s * d1.x(),
            p1.y() + s * d1.y(),
            p1.z() + s * d1.z(),
        );
        let point2 = Point3D::new(
            p3.x() + t * d2.x(),
            p3.y() + t * d2.y(),
            p3.z() + t * d2.z(),
        );

        if point1.distance_to(&point2) <= tolerance {
            Some(point1)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn infinite_line3d_infinite_line3d_intersection<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let point = line_line_intersection_raw(line1, line2)?;
    // 交点候補が両直線上に乗っているか数値誤差で確認
    if line2.distance_to_point(&point) <= tolerance {
        Some(point)
    } else {
        None
    }
}

pub fn infinite_line3d_line_segment3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let segment_line = segment.line();
    let point = line_line_intersection_raw(line, segment_line)?;
    if segment.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn infinite_line3d_ray3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;
    let point = line_line_intersection_raw(line, &ray_line)?;
    if ray.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn plane3d_line_segment3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let direction = Vector3D::from_points(&start, &end);

    if direction.is_zero() {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= T::EPSILON {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&start, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    if t >= T::ZERO && t <= T::ONE {
        Some(Point3D::new(
            start.x() + t * direction.x(),
            start.y() + t * direction.y(),
            start.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_ray3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let direction = ray.direction_vector();
    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= T::EPSILON {
        return if plane.contains_point(origin, tolerance) {
            Some(origin)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&origin, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    if t >= T::ZERO {
        Some(Point3D::new(
            origin.x() + t * direction.x(),
            origin.y() + t * direction.y(),
            origin.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_infinite_line3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let lp = line.point();
    let ld = line.direction();
    let line_point = Point3D::new(lp.0, lp.1, lp.2);
    let line_dir_vec = Vector3D::new(ld.0, ld.1, ld.2);
    let normal = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.distance_to_point(line_point).abs() <= tolerance {
            Some(line_point)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&line_point, &plane.origin());
    let t = to_plane.dot(&normal) / denom;
    Some(Point3D::new(
        line_point.x() + t * line_dir_vec.x(),
        line_point.y() + t * line_dir_vec.y(),
        line_point.z() + t * line_dir_vec.z(),
    ))
}

pub fn infinite_line3d_spherical_surface3d_intersections<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    let mut result = Vec::new();

    let center_tuple = sphere.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = sphere.radius();

    let pt = line.point();
    let dir_tuple = line.direction();
    let start = Point3D::new(pt.0, pt.1, pt.2);
    let d = Vector3D::new(dir_tuple.0, dir_tuple.1, dir_tuple.2);
    let f = Vector3D::from_points(&center, &start);

    let a = d.dot(&d);
    if a.abs() <= T::EPSILON {
        return result;
    }

    let b = (f.dot(&d)) * (T::ONE + T::ONE);
    let c = f.dot(&f) - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    let p1 = Point3D::new(
        start.x() + t1 * d.x(),
        start.y() + t1 * d.y(),
        start.z() + t1 * d.z(),
    );
    result.push(p1);

    if (t2 - t1).abs() > T::EPSILON {
        let p2 = Point3D::new(
            start.x() + t2 * d.x(),
            start.y() + t2 * d.y(),
            start.z() + t2 * d.z(),
        );
        result.push(p2);
    }

    result
}

pub fn ray3d_spherical_surface3d_intersections<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    let mut result = Vec::new();

    let center_tuple = sphere.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = sphere.radius();

    let start = ray.origin();
    let d = ray.direction_vector();
    let f = Vector3D::from_points(&center, &start);

    let a = d.dot(&d);
    if a.abs() <= T::EPSILON {
        return result;
    }

    let b = (f.dot(&d)) * (T::ONE + T::ONE);
    let c = f.dot(&f) - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    // Ray は t >= 0 のみ有効
    if t1 >= T::ZERO {
        let p1 = Point3D::new(
            start.x() + t1 * d.x(),
            start.y() + t1 * d.y(),
            start.z() + t1 * d.z(),
        );
        result.push(p1);
    }

    if t2 >= T::ZERO && (t2 - t1).abs() > T::EPSILON {
        let p2 = Point3D::new(
            start.x() + t2 * d.x(),
            start.y() + t2 * d.y(),
            start.z() + t2 * d.z(),
        );
        result.push(p2);
    }

    result
}

pub fn ray3d_ray3d_intersection<T: Scalar>(
    ray1: &Ray3D<T>,
    ray2: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let p1 = ray1.origin();
    let p2 = ray2.origin();
    let d1 = ray1.direction_vector();
    let d2 = ray2.direction_vector();

    let r = Vector3D::from_points(&p2, &p1);

    let a = d1.dot(&d1);
    let b = d1.dot(&d2);
    let c = d2.dot(&d2);
    let d = d1.dot(&r);
    let e = d2.dot(&r);

    let denom = a * c - b * b;
    if denom.abs() <= tolerance {
        return None; // 平行
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    if s < T::ZERO || t < T::ZERO {
        return None; // Ray 範囲外
    }

    let point1 = ray1.point_at_parameter(s);
    let point2 = ray2.point_at_parameter(t);

    if point1.distance_to(&point2) <= tolerance {
        Some(point1)
    } else {
        None
    }
}

pub fn ray3d_line_segment3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;
    let segment_line = segment.line();
    let point = line_line_intersection_raw(&ray_line, segment_line)?;
    if ray.contains_point(&point, tolerance) && segment.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn ray3d_infinite_line3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;
    let point = line_line_intersection_raw(&ray_line, line)?;
    if ray.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn triangle3d_ray3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let va = triangle.vertex_a();
    let vb = triangle.vertex_b();
    let vc = triangle.vertex_c();

    let v0 = Point3D::new(va.0, va.1, va.2);
    let v1 = Point3D::new(vb.0, vb.1, vb.2);
    let v2 = Point3D::new(vc.0, vc.1, vc.2);

    let edge1 = Vector3D::from_points(&v0, &v1);
    let edge2 = Vector3D::from_points(&v0, &v2);

    let ray_dir = ray.direction_vector();
    let h = ray_dir.cross(&edge2);
    let a = edge1.dot(&h);

    if a.abs() <= tolerance {
        return None;
    }

    let f = T::ONE / a;
    let s = Vector3D::from_points(&v0, &ray.origin());
    let u = f * s.dot(&h);

    if u < T::ZERO || u > T::ONE {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray_dir.dot(&q);

    if v < T::ZERO || u + v > T::ONE {
        return None;
    }

    let t = f * edge2.dot(&q);
    if t < T::ZERO {
        return None;
    }

    Some(Point3D::new(
        ray.origin().x() + t * ray_dir.x(),
        ray.origin().y() + t * ray_dir.y(),
        ray.origin().z() + t * ray_dir.z(),
    ))
}

pub fn triangle3d_line_segment3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // 線分をRayに変換してMöller-Trumboreで交点を計算し、線分範囲内かチェック
    let start = segment.start();
    let end = segment.end();
    let dir = end - start;
    let length = dir.length();
    if length <= tolerance {
        return None;
    }
    let ray = Ray3D::new(start, dir)?;
    let point = triangle3d_ray3d_intersection(triangle, &ray, tolerance)?;

    // 線分の範囲内（0 <= t <= length）かチェック
    let t = ray.parameter_for_point(&point);
    if t >= T::ZERO && t <= length {
        Some(point)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        arc2d_circle2d_intersections, circle2d_circle2d_intersections,
        circle2d_line_segment2d_intersections, infinite_line3d_infinite_line3d_intersection,
        infinite_line3d_line_segment3d_intersection, infinite_line3d_ray3d_intersection,
        infinite_line3d_spherical_surface3d_intersections, line_segment2d_arc2d_intersections,
        line_segment2d_circle2d_intersections, line_segment2d_line_segment2d_intersection,
        line_segment3d_line_segment3d_intersection,
        line_segment3d_spherical_surface3d_intersections, ray3d_spherical_surface3d_intersections,
    };
    use super::{
        ray3d_infinite_line3d_intersection, ray3d_line_segment3d_intersection,
        ray3d_ray3d_intersection, triangle3d_line_segment3d_intersection,
        triangle3d_ray3d_intersection,
    };
    use crate::{
        Angle, Arc2D, Circle2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Point2D, Point3D,
        Ray3D, SphericalSurface3D, Triangle3D, Vector3D,
    };

    #[test]
    fn circle_circle_returns_two_points() {
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();

        let points = circle2d_circle2d_intersections(&c1, &c2, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn circle_circle_tangent_with_small_gap_uses_tolerance() {
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let c2 = Circle2D::new(
            Point2D::new(2.0 + super::SMALL_GAP_WITHIN_TOLERANCE_F64, 0.0),
            1.0,
        )
        .unwrap();

        let points = circle2d_circle2d_intersections(&c1, &c2, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn circle_line_segment_returns_two_points() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let points = circle2d_line_segment2d_intersections(
            &circle,
            &segment,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment_circle_returns_two_points() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let points = line_segment2d_circle2d_intersections(
            &segment,
            &circle,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment_arc_filters_outside_angle_range() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(
            circle,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI * 0.5),
        )
        .unwrap();

        let points =
            line_segment2d_arc2d_intersections(&segment, &arc, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn arc_circle_filters_outside_angle_range() {
        let arc_circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let arc = Arc2D::new(
            arc_circle,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI * 0.5),
        )
        .unwrap();
        let circle = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();

        let points =
            arc2d_circle2d_intersections(&arc, &circle, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn line_segment_line_segment_returns_single_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0)).unwrap();

        let p = line_segment2d_line_segment2d_intersection(
            &seg1,
            &seg2,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
    }

    #[test]
    fn line_segment3d_spherical_surface_returns_two_points() {
        let segment =
            LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();

        let points = line_segment3d_spherical_surface3d_intersections(&segment, &sphere);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment3d_line_segment3d_returns_single_intersection() {
        let seg1 =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)).unwrap();
        let seg2 =
            LineSegment3D::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();

        let p = line_segment3d_line_segment3d_intersection(
            &seg1,
            &seg2,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
    }

    #[test]
    fn infinite_line3d_spherical_surface_returns_two_points() {
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-2.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        )
        .unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();

        let points = infinite_line3d_spherical_surface3d_intersections(&line, &sphere);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn infinite_line3d_spherical_surface_tangent_returns_one_point() {
        // 球の赤道に接する直線（y=1 の XZ方向）
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-2.0, 1.0, 0.0),
            Point3D::new(2.0, 1.0, 0.0),
        )
        .unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();

        let points = infinite_line3d_spherical_surface3d_intersections(&line, &sphere);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn ray3d_spherical_surface_returns_two_points() {
        // 原点から-x方向に発射、球（中心原点, r=1）を貫通
        let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();

        let points = ray3d_spherical_surface3d_intersections(&ray, &sphere);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn ray3d_spherical_surface_behind_origin_returns_zero_points() {
        // 球から離れる方向に発射 → 交点なし
        let ray = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();

        let points = ray3d_spherical_surface3d_intersections(&ray, &sphere);
        assert_eq!(points.len(), 0);
    }

    #[test]
    fn infinite_line3d_infinite_line3d_returns_intersection() {
        // XY平面上でX軸とY軸が交わる
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, -1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let p = infinite_line3d_infinite_line3d_intersection(
            &line1,
            &line2,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
        let p = p.unwrap();
        assert!(p.x().abs() < super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.y().abs() < super::STANDARD_TEST_TOLERANCE_F64);
    }

    #[test]
    fn infinite_line3d_infinite_line3d_parallel_returns_none() {
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        )
        .unwrap();

        let p = infinite_line3d_infinite_line3d_intersection(
            &line1,
            &line2,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_none());
    }

    #[test]
    fn infinite_line3d_line_segment3d_returns_intersection() {
        // X軸の無限直線と、(0,-1,0)-(0,1,0) の線分が原点で交わる
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let segment =
            LineSegment3D::new(Point3D::new(0.0, -1.0, 0.0), Point3D::new(0.0, 1.0, 0.0)).unwrap();

        let p = infinite_line3d_line_segment3d_intersection(
            &line,
            &segment,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
    }

    #[test]
    fn infinite_line3d_line_segment3d_outside_segment_returns_none() {
        // X軸の無限直線と、y=2 上の線分（交点が線分外）
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(0.0, 3.0, 0.0)).unwrap();

        let p = infinite_line3d_line_segment3d_intersection(
            &line,
            &segment,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_none());
    }

    #[test]
    fn infinite_line3d_ray3d_returns_intersection() {
        // X軸の無限直線と、(0,-2,0) から +y 方向の Ray が原点で交わる
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray = Ray3D::new(Point3D::new(0.0, -2.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let p = infinite_line3d_ray3d_intersection(&line, &ray, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_some());
    }

    #[test]
    fn infinite_line3d_ray3d_behind_ray_returns_none() {
        // X軸の無限直線と、(0,-2,0) から -y 方向の Ray（交点が後方）→ None
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray = Ray3D::new(Point3D::new(0.0, -2.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();

        let p = infinite_line3d_ray3d_intersection(&line, &ray, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_none());
    }

    #[test]
    fn ray3d_ray3d_returns_intersection() {
        // (+x 方向) と (+y 方向) の Ray が原点で交わる
        let ray1 = Ray3D::new(Point3D::new(-1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray2 = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let p = ray3d_ray3d_intersection(&ray1, &ray2, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_some());
        let p = p.unwrap();
        assert!(p.x().abs() < super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.y().abs() < super::STANDARD_TEST_TOLERANCE_F64);
    }

    #[test]
    fn ray3d_ray3d_behind_origin_returns_none() {
        // Ray が互いに反対方向を向いている → 交点なし
        let ray1 = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray2 = Ray3D::new(Point3D::new(0.0, 1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let p = ray3d_ray3d_intersection(&ray1, &ray2, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_none());
    }

    #[test]
    fn ray3d_line_segment3d_returns_intersection() {
        // +y 方向 Ray と y 軸上の線分 (-1,0,0)-(1,0,0) が原点で交わる
        let ray = Ray3D::new(Point3D::new(0.0, -2.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        let segment =
            LineSegment3D::new(Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();

        let p =
            ray3d_line_segment3d_intersection(&ray, &segment, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_some());
    }

    #[test]
    fn ray3d_infinite_line3d_returns_intersection() {
        // +y 方向 Ray と X 軸の無限直線が原点で交わる
        let ray = Ray3D::new(Point3D::new(0.0, -2.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let p = ray3d_infinite_line3d_intersection(&ray, &line, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_some());
    }

    #[test]
    fn triangle3d_ray3d_returns_intersection() {
        // XY平面上の三角形に +z から Ray を当てる → 交点あり
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let ray = Ray3D::new(Point3D::new(0.2, 0.2, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();

        let p = triangle3d_ray3d_intersection(&triangle, &ray, super::STANDARD_TEST_TOLERANCE_F64);
        assert!(p.is_some());
        assert!(p.unwrap().z().abs() < super::STANDARD_TEST_TOLERANCE_F64);
    }

    #[test]
    fn triangle3d_line_segment3d_returns_intersection() {
        // XY平面上の三角形を z=-1→z=1 の線分が貫通する
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let segment =
            LineSegment3D::new(Point3D::new(0.2, 0.2, -1.0), Point3D::new(0.2, 0.2, 1.0)).unwrap();

        let p = triangle3d_line_segment3d_intersection(
            &triangle,
            &segment,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
    }
}

#[test]
fn plane3d_line_segment3d_returns_intersection() {
    // z=0 の XY平面と z=-1→z=1 の線分 → z=0 で交差
    let plane = Plane3D::xy_plane(0.0_f64);
    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();
    let p = plane3d_line_segment3d_intersection(&plane, &segment, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_some());
    assert!(p.unwrap().z().abs() < STANDARD_TEST_TOLERANCE_F64);
}

#[test]
fn plane3d_line_segment3d_parallel_returns_none() {
    // z=0 の XY平面と z=1 上の水平線分 → 平行で交点なし
    let plane = Plane3D::xy_plane(0.0_f64);
    let segment =
        LineSegment3D::new(Point3D::new(-1.0, 0.0, 1.0), Point3D::new(1.0, 0.0, 1.0)).unwrap();
    let p = plane3d_line_segment3d_intersection(&plane, &segment, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_none());
}

#[test]
fn plane3d_ray3d_returns_intersection() {
    let plane = Plane3D::xy_plane(0.0_f64);
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, -2.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let p = plane3d_ray3d_intersection(&plane, &ray, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_some());
}

#[test]
fn plane3d_ray3d_behind_ray_returns_none() {
    // 平面より先にある Ray が平面から遠ざかる方向 → None
    let plane = Plane3D::xy_plane(0.0_f64);
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 2.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let p = plane3d_ray3d_intersection(&plane, &ray, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_none());
}

#[test]
fn plane3d_infinite_line3d_returns_intersection() {
    let plane = Plane3D::xy_plane(0.0_f64);
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0))
            .unwrap();
    let p = plane3d_infinite_line3d_intersection(&plane, &line, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_some());
}

#[test]
fn plane3d_infinite_line3d_parallel_returns_none() {
    let plane = Plane3D::xy_plane(0.0_f64);
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(-1.0, 0.0, 1.0), Point3D::new(1.0, 0.0, 1.0))
            .unwrap();
    let p = plane3d_infinite_line3d_intersection(&plane, &line, STANDARD_TEST_TOLERANCE_F64);
    assert!(p.is_none());
}
