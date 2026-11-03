//! 2D の形状同士の交差

use crate::{
    Angle, Circle2D, Ellipse2D, InfiniteLine2D, IntersectionGeometry, IntersectionResult,
    IntersectionTopology, LineSegment2D, Point2D, Ray2D, Triangle2D, Vector2D,
};
use geo_contracts::{
    Circle2DProperties, Ellipse2DEvaluation, Ellipse2DProperties, InfiniteLine2DDistance,
    InfiniteLine2DProjection, InfiniteLine2DProperties, LineSegment2DConstructor,
    LineSegment2DProperties, Ray2DProperties, Scalar, Triangle2DBoundaryAccess,
};

pub fn ray2d_line_segment2d_intersection<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let s1 = segment.start_point();
    let s2 = segment.end_point();

    let origin = ray.origin();
    let direction = ray.direction().as_vector();

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

pub fn ray2d_circle2d_intersections<T: Scalar>(
    ray: &Ray2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = circle.center();
    let origin = ray.origin();
    let direction = ray.direction().as_vector();

    let oc = origin - center;
    let dir_vec = direction;

    let a = dir_vec.dot(dir_vec);
    let b = (oc.dot(dir_vec)) * (T::ONE + T::ONE);
    let c = oc.dot(oc) - circle.radius() * circle.radius();

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

pub fn infinite_line2d_circle2d_intersections<T: Scalar>(
    line: &InfiniteLine2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = circle.center();
    let projected = line.closest_point(center);
    let dist_to_center = line.distance_to_point(center);
    let radius = circle.radius();
    if dist_to_center > radius + tolerance {
        return IntersectionResult::from_option_points2d(Vec::new(), false, tolerance);
    }
    if (dist_to_center - radius).abs() <= tolerance {
        return IntersectionResult::from_option_points2d(vec![projected], false, tolerance);
    }
    let half_chord = (radius * radius - dist_to_center * dist_to_center).sqrt();
    let dir = InfiniteLine2DProperties::direction(line);
    let (dx, dy) = (dir.x(), dir.y());
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

pub fn infinite_line2d_line_segment2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let segment_start = LineSegment2DProperties::start_point(segment);
    let (s1x, s1y) = (segment_start.x(), segment_start.y());
    let segment_end = LineSegment2DProperties::end_point(segment);
    let (s2x, s2y) = (segment_end.x(), segment_end.y());
    let line_point = InfiniteLine2DProperties::point(line);
    let line_dir = InfiniteLine2DProperties::direction(line);
    let (ldx, ldy) = (line_dir.x(), line_dir.y());
    let dx_seg = s2x - s1x;
    let dy_seg = s2y - s1y;
    let denominator = ldx * dy_seg - ldy * dx_seg;
    if denominator.abs() <= tolerance {
        let line_dir = line_dir.as_vector();
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
    let t2 = ((s1x - line_point.x()) * ldy - (s1y - line_point.y()) * ldx) / denominator;
    let opt = if t2 >= T::ZERO - tolerance && t2 <= T::ONE + tolerance {
        Some(Point2D::new(s1x + t2 * dx_seg, s1y + t2 * dy_seg))
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

pub fn infinite_line2d_ray2d_intersection<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let ray_origin = Ray2DProperties::origin(ray);
    let (ox, oy) = (ray_origin.x(), ray_origin.y());
    let ray_dir = Ray2DProperties::direction(ray);
    let (rdx, rdy) = (ray_dir.x(), ray_dir.y());
    let line_point = InfiniteLine2DProperties::point(line);
    let line_dir = InfiniteLine2DProperties::direction(line);
    let (ldx, ldy) = (line_dir.x(), line_dir.y());
    let denominator = ldx * rdy - ldy * rdx;
    if denominator.abs() <= tolerance {
        let line_dir = line_dir.as_vector();
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
    let t2 = ((ox - line_point.x()) * ldy - (oy - line_point.y()) * ldx) / denominator;
    let opt = if t2 >= T::ZERO - tolerance {
        Some(Point2D::new(ox + t2 * rdx, oy + t2 * rdy))
    } else {
        None
    };
    IntersectionResult::from_option_point2d(opt, false, tolerance)
}

/// 円と楕円の交点
///
/// 円の中心までの距離が極値になる楕円上の点のうち、`tolerance` の範囲で円上にある点を接する点とする。
/// 楕円のパラメータについての 4 次多項式の根のうち、接する点から `tolerance` より離れた点を横断する交点とする。
/// 円と楕円が一致する場合は `Coincident` を返す。
pub fn circle2d_ellipse2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    ellipse: &Ellipse2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = circle.center();
    let radius = circle.radius();
    let (a, b) = (ellipse.semi_major_axis(), ellipse.semi_minor_axis());
    let (u, v) = ellipse2d_local_coordinates(ellipse.center(), ellipse.rotation(), center);
    if (a - b).abs() <= tolerance
        && (u * u + v * v).sqrt() <= tolerance
        && (a - radius).abs() <= tolerance
    {
        return IntersectionResult::new(
            IntersectionGeometry::Coincident,
            IntersectionTopology::Coincident,
            false,
            tolerance,
        );
    }

    let distinct_from = |point: Point2D<T>, points: &[Point2D<T>]| {
        points
            .iter()
            .all(|other| other.distance_to(point) > tolerance)
    };
    let mut tangent_points: Vec<Point2D<T>> = Vec::new();
    for parameter in geo_commons::ellipse_2d_distance_critical_parameters(u, v, a, b) {
        let point = ellipse.point_at_parameter(parameter);
        if (point.distance_to(center) - radius).abs() <= tolerance
            && distinct_from(point, &tangent_points)
        {
            tangent_points.push(point);
        }
    }
    let mut points = tangent_points.clone();
    for parameter in geo_commons::ellipse_2d_circle_intersection_parameters(u, v, a, b, radius) {
        let point = ellipse.point_at_parameter(parameter);
        if distinct_from(point, &points) {
            points.push(point);
        }
    }
    let is_tangent = points.len() == tangent_points.len();
    IntersectionResult::from_option_points2d(points, is_tangent, tolerance)
}

/// 楕円の中心を原点とし、長軸を x 軸とする局所座標での点の座標
pub(crate) fn ellipse2d_local_coordinates<T: Scalar>(
    ellipse_center: Point2D<T>,
    rotation: Angle<T>,
    point: Point2D<T>,
) -> (T, T) {
    let offset = Vector2D::from_points(ellipse_center, point);
    let (cos, sin) = (rotation.cos(), rotation.sin());
    (
        offset.x() * cos + offset.y() * sin,
        -offset.x() * sin + offset.y() * cos,
    )
}

/// 三角形（領域）と線分の重なり
///
/// 線分を三角形の各辺の内側の半平面で切り取る（凸な領域による切り取り）。
/// 三角形で切り取った区間が `tolerance` より長ければ重なる線分を返す。そうでなく、三角形を `tolerance` だけ
/// 広げた領域と重なる場合は、その区間の中点を接する点として返す。
pub fn triangle2d_line_segment2d_intersection<T: Scalar>(
    triangle: &Triangle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let start = segment.start_point();
    let direction = Vector2D::from_points(start, segment.end_point());
    let point_at = |parameter: T| start + direction * parameter;

    if let Some((enter, exit)) = clip_segment_by_triangle(triangle, start, direction, T::ZERO) {
        let (overlap_start, overlap_end) = (point_at(enter), point_at(exit));
        if overlap_start.distance_to(overlap_end) > tolerance {
            if let Some(overlap) = LineSegment2D::new(overlap_start, overlap_end) {
                return IntersectionResult::new(
                    IntersectionGeometry::Segment2D(overlap),
                    IntersectionTopology::Crossing,
                    false,
                    tolerance,
                );
            }
        }
    }
    let touching = clip_segment_by_triangle(triangle, start, direction, tolerance)
        .map(|(enter, exit)| point_at((enter + exit) / (T::ONE + T::ONE)));
    IntersectionResult::from_option_point2d(touching, true, tolerance)
}

/// 線分 `start + direction * t`（`0 <= t <= 1`）のうち、三角形を `margin` だけ広げた領域に入るパラメータの区間
fn clip_segment_by_triangle<T: Scalar>(
    triangle: &Triangle2D<T>,
    start: Point2D<T>,
    direction: Vector2D<T>,
    margin: T,
) -> Option<(T, T)> {
    let vertices = [
        triangle.vertex_a(),
        triangle.vertex_b(),
        triangle.vertex_c(),
    ];
    let counterclockwise = Vector2D::from_points(vertices[0], vertices[1])
        .cross(Vector2D::from_points(vertices[0], vertices[2]))
        > T::ZERO;

    let mut enter = T::ZERO;
    let mut exit = T::ONE;
    for i in 0..3 {
        let edge_start = vertices[i];
        let edge = Vector2D::from_points(edge_start, vertices[(i + 1) % 3]);
        let edge_length = edge.length();
        // 辺から三角形の内側へ向かう符号付き距離
        let inward = |vector: Vector2D<T>| {
            let value = edge.cross(vector) / edge_length;
            if counterclockwise {
                value
            } else {
                -value
            }
        };
        let offset = inward(Vector2D::from_points(edge_start, start)) + margin;
        let rate = inward(direction);
        if rate == T::ZERO {
            if offset < T::ZERO {
                return None;
            }
            continue;
        }
        let parameter = -offset / rate;
        if rate > T::ZERO {
            enter = enter.max(parameter);
        } else {
            exit = exit.min(parameter);
        }
        if enter > exit {
            return None;
        }
    }
    Some((enter, exit))
}

fn is_point_on_line<T: Scalar>(
    line_point: Point2D<T>,
    line_dir: Vector2D<T>,
    p: Point2D<T>,
    tolerance: T,
) -> bool {
    Vector2D::from_points(line_point, p).cross(line_dir).abs() <= tolerance
}

fn collinear_ray_segment_overlap_result<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let origin = ray.origin();
    let direction = ray.direction().as_vector();
    let s1 = segment.start_point();
    let s2 = segment.end_point();
    let dir_norm = direction.length_squared();

    if dir_norm <= tolerance * tolerance {
        return IntersectionResult::disjoint(tolerance);
    }

    let t1 = Vector2D::from_points(origin, s1).dot(direction) / dir_norm;
    let t2 = Vector2D::from_points(origin, s2).dot(direction) / dir_norm;
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

#[cfg(test)]
mod tests {
    use super::{
        infinite_line2d_line_segment2d_intersection, infinite_line2d_ray2d_intersection,
        ray2d_circle2d_intersections, ray2d_line_segment2d_intersection,
    };
    use crate::{
        Circle2D, InfiniteLine2D, IntersectionGeometry, IntersectionTopology, LineSegment2D,
        Point2D, Ray2D, Vector2D,
    };
    use analysis::test_constants;
    use geo_contracts::Circle2DConstructor;

    use geo_contracts::InfiniteLine2DConstructor;
    use geo_contracts::LineSegment2DConstructor;

    use geo_contracts::Ray2DConstructor;

    const STANDARD_TEST_TOLERANCE_F64: f64 = test_constants::DISTANCE_TOLERANCE_F64;

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
}
