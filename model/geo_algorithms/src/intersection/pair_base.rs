//! 交点計算のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の幾何計算を集約する。

use geo_foundation::{
    Arc2DProperties, Circle2DProperties, InfiniteLine3DProperties, LineSegment2DProperties, Scalar,
    SphericalSurface3DProperties,
};
use geo_primitives::{
    Arc2D, Circle2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Point2D, Point3D, Ray3D,
    SphericalSurface3D, Vector3D,
};

pub fn circle2d_circle2d_intersections<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center1 = circle1.center();
    let center2 = circle2.center();
    let r1 = circle1.radius();
    let r2 = circle2.radius();

    let dx = center2.0 - center1.0;
    let dy = center2.1 - center1.1;
    let d = (dx * dx + dy * dy).sqrt();

    if d > r1 + r2 || d < (r1 - r2).abs() || d == T::ZERO {
        return result;
    }

    let a = (r1 * r1 - r2 * r2 + d * d) / ((T::ONE + T::ONE) * d);
    let h_squared = r1 * r1 - a * a;
    if h_squared < T::ZERO {
        return result;
    }

    let h = h_squared.sqrt();
    let px = center1.0 + a * dx / d;
    let py = center1.1 + a * dy / d;

    if h == T::ZERO {
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
) -> Vec<Point2D<T>> {
    line_segment2d_circle2d_intersections(segment, circle)
}

pub fn arc2d_circle2d_intersections<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let base_circle = Circle2D::new(
        Point2D::new(center_x, center_y),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        circle2d_circle2d_intersections(&base_circle, circle)
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

    let a = dx * dx + dy * dy;
    if a.abs() <= T::EPSILON {
        return result;
    }

    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= T::ZERO && t1 <= T::ONE {
        let x = start.0 + t1 * dx;
        let y = start.1 + t1 * dy;
        result.push(Point2D::new(x, y));
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > T::EPSILON {
        let x = start.0 + t2 * dx;
        let y = start.1 + t2 * dy;
        result.push(Point2D::new(x, y));
    }

    result
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(
        Point2D::new(arc.center().0, arc.center().1),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle)
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
    if denominator.abs() < T::EPSILON {
        return None;
    }

    let t1 = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denominator;
    let t2 = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denominator;

    if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
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

#[cfg(test)]
mod tests {
    use super::{
        arc2d_circle2d_intersections, circle2d_circle2d_intersections,
        circle2d_line_segment2d_intersections, infinite_line3d_spherical_surface3d_intersections,
        line_segment2d_arc2d_intersections, line_segment2d_circle2d_intersections,
        line_segment2d_line_segment2d_intersection, line_segment3d_line_segment3d_intersection,
        line_segment3d_spherical_surface3d_intersections, ray3d_spherical_surface3d_intersections,
    };
    use geo_primitives::{
        Angle, Arc2D, Circle2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Point2D, Point3D,
        Ray3D, SphericalSurface3D, Vector3D,
    };

    #[test]
    fn circle_circle_returns_two_points() {
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();

        let points = circle2d_circle2d_intersections(&c1, &c2);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn circle_line_segment_returns_two_points() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let points = circle2d_line_segment2d_intersections(&circle, &segment);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment_circle_returns_two_points() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let points = line_segment2d_circle2d_intersections(&segment, &circle);
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

        let points = line_segment2d_arc2d_intersections(&segment, &arc);
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

        let points = arc2d_circle2d_intersections(&arc, &circle);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn line_segment_line_segment_returns_single_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0)).unwrap();

        let p = line_segment2d_line_segment2d_intersection(&seg1, &seg2);
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

        let p = line_segment3d_line_segment3d_intersection(&seg1, &seg2, 1e-6);
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
}
