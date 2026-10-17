//! 形状をまたぐ演算の網羅表（`dev/architecture/CROSS_OPERATION_COVERAGE_MATRIX_SPEC.md`）
//!
//! 実装済みの組み合わせは trait の実装をコンパイル時に確かめ、代表的な配置で結果を確かめる。
//! 未対応の組み合わせは仕様書の未対応表で管理する。

use geo_algorithms::{
    Aabb3D, Angle, Arc2D, Circle2D, Circle3D, ConicalSolid3D, ConicalSurface3D, CrossCollision,
    CrossDistance, CrossIntersection, CylindricalSolid3D, Direction3D, Ellipse2D, EllipseArc2D,
    EllipsoidalSolid3D, InfiniteLine2D, InfiniteLine3D, IntersectionResult, LineSegment2D,
    LineSegment3D, NurbsCurve3D, Plane3D, Point2D, Point3D, Ray2D, Ray3D, SphericalSolid3D,
    SphericalSurface3D, Triangle2D, Triangle3D, Vector2D, Vector3D,
};
use geo_contracts::{
    Arc2DConstructor, Circle2DConstructor, ConicalSolid3DConstructor, ConicalSurface3DConstructor,
    Ellipse2DConstructor, EllipseArc2DConstructor, InfiniteLine2DConstructor,
    InfiniteLine3DConstructor, LineSegment2DConstructor, LineSegment3DConstructor,
    Plane3DConstructor, Ray2DConstructor, Ray3DConstructor, SphericalSolid3DConstructor,
    SphericalSurface3DConstructor, Triangle2DConstructor, Triangle3DConstructor,
};

const TOLERANCE: f64 = analysis::test_constants::DISTANCE_TOLERANCE_F64;

fn collision<A: CrossCollision<f64, B>, B>() {}
fn distance<A: CrossDistance<f64, B>, B>() {}
fn intersection<A: CrossIntersection<f64, B>, B>() {}

/// A-B と B-A の両方を実装していることを確かめる
macro_rules! symmetric {
    ($operation:ident, $a:ident, $b:ident) => {
        $operation::<$a<f64>, $b<f64>>();
        $operation::<$b<f64>, $a<f64>>();
    };
}

/// A-B の向きを実装していることを確かめる
macro_rules! one_way {
    ($operation:ident, $a:ident, $b:ident) => {
        $operation::<$a<f64>, $b<f64>>();
    };
}

#[test]
fn implemented_2d_pairs() {
    symmetric!(collision, Ray2D, Circle2D);
    symmetric!(collision, Ray2D, LineSegment2D);
    symmetric!(collision, LineSegment2D, Arc2D);
    symmetric!(collision, Arc2D, Circle2D);
    symmetric!(collision, Triangle2D, Circle2D);
    symmetric!(collision, Triangle2D, LineSegment2D);
    symmetric!(collision, Ellipse2D, Circle2D);
    symmetric!(collision, EllipseArc2D, Circle2D);
    symmetric!(collision, InfiniteLine2D, Circle2D);
    symmetric!(collision, InfiniteLine2D, LineSegment2D);
    symmetric!(collision, InfiniteLine2D, Ray2D);

    symmetric!(intersection, Circle2D, Ellipse2D);
    symmetric!(intersection, Triangle2D, LineSegment2D);
    symmetric!(intersection, Ray2D, Circle2D);
    symmetric!(intersection, Ray2D, LineSegment2D);
    symmetric!(intersection, InfiniteLine2D, Circle2D);
    symmetric!(intersection, InfiniteLine2D, LineSegment2D);
    symmetric!(intersection, InfiniteLine2D, Ray2D);
}

#[test]
fn implemented_3d_pairs() {
    symmetric!(collision, InfiniteLine3D, LineSegment3D);
    symmetric!(collision, InfiniteLine3D, Ray3D);

    one_way!(distance, InfiniteLine3D, InfiniteLine3D);
    one_way!(distance, LineSegment3D, Aabb3D);
    symmetric!(distance, NurbsCurve3D, Circle3D);
    symmetric!(distance, NurbsCurve3D, CylindricalSolid3D);
    symmetric!(distance, NurbsCurve3D, EllipsoidalSolid3D);
    symmetric!(distance, NurbsCurve3D, InfiniteLine3D);
    symmetric!(distance, NurbsCurve3D, LineSegment3D);
    symmetric!(distance, NurbsCurve3D, Plane3D);
    symmetric!(distance, NurbsCurve3D, Ray3D);
    symmetric!(distance, NurbsCurve3D, SphericalSolid3D);

    one_way!(intersection, InfiniteLine3D, LineSegment3D);
    symmetric!(intersection, InfiniteLine3D, Ray3D);
    one_way!(intersection, InfiniteLine3D, InfiniteLine3D);
    one_way!(intersection, LineSegment3D, LineSegment3D);
    one_way!(intersection, Ray3D, Ray3D);
    one_way!(intersection, Plane3D, LineSegment3D);
    one_way!(intersection, Plane3D, Ray3D);
    symmetric!(intersection, Triangle3D, LineSegment3D);
    symmetric!(intersection, Triangle3D, Ray3D);
    one_way!(intersection, ConicalSolid3D, Point3D);
    one_way!(intersection, ConicalSolid3D, InfiniteLine3D);
    one_way!(intersection, ConicalSolid3D, Ray3D);
    one_way!(intersection, ConicalSolid3D, LineSegment3D);
    one_way!(intersection, ConicalSurface3D, Point3D);
    one_way!(intersection, ConicalSurface3D, InfiniteLine3D);
    symmetric!(intersection, ConicalSurface3D, Ray3D);
    one_way!(intersection, ConicalSurface3D, LineSegment3D);
    one_way!(intersection, SphericalSolid3D, Point3D);
    one_way!(intersection, SphericalSolid3D, InfiniteLine3D);
    one_way!(intersection, SphericalSolid3D, Ray3D);
    symmetric!(intersection, SphericalSurface3D, Ray3D);
    one_way!(intersection, LineSegment3D, SphericalSurface3D);
}

fn point_count(result: &IntersectionResult<f64>) -> usize {
    use geo_algorithms::IntersectionGeometry;
    match &result.geometry {
        IntersectionGeometry::None => 0,
        IntersectionGeometry::Point(_) | IntersectionGeometry::Point2D(_) => 1,
        IntersectionGeometry::Points(points) => points.len(),
        IntersectionGeometry::Points2D(points) => points.len(),
        _ => usize::MAX,
    }
}

#[test]
fn collision_2d_detects_contact_in_both_directions() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
    let toward = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
    let above = Ray2D::new(Point2D::new(-2.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();
    assert!(toward.collides(&circle, TOLERANCE));
    assert!(circle.collides(&toward, TOLERANCE));
    assert!(!above.collides(&circle, TOLERANCE));

    let line = InfiniteLine2D::new(Point2D::new(0.0, 0.5), Vector2D::new(1.0, 0.0)).unwrap();
    let far_line = InfiniteLine2D::new(Point2D::new(0.0, 3.0), Vector2D::new(1.0, 0.0)).unwrap();
    assert!(line.collides(&circle, TOLERANCE));
    assert!(!far_line.collides(&circle, TOLERANCE));

    let segment = LineSegment2D::new(Point2D::new(0.0, -1.0), Point2D::new(0.0, 1.0)).unwrap();
    assert!(toward.collides(&segment, TOLERANCE));
    assert!(segment.collides(&toward, TOLERANCE));
    assert!(!above.collides(&segment, TOLERANCE));
    assert!(!far_line.collides(&toward, TOLERANCE));
    assert!(!toward.collides(&far_line, TOLERANCE));
    assert!(
        InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0))
            .unwrap()
            .collides(&segment, TOLERANCE)
    );

    let upper_arc = Arc2D::new(
        Point2D::new(0.0, 0.0),
        1.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(180.0),
    )
    .unwrap();
    let lower_segment =
        LineSegment2D::new(Point2D::new(-2.0, -0.5), Point2D::new(2.0, -0.5)).unwrap();
    assert!(segment.collides(&upper_arc, TOLERANCE));
    assert!(upper_arc.collides(&segment, TOLERANCE));
    assert!(!lower_segment.collides(&upper_arc, TOLERANCE));
}

#[test]
fn collision_2d_arc_and_circle_follow_the_arc_range() {
    let upper_arc = Arc2D::new(
        Point2D::new(0.0, 0.0),
        1.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(180.0),
    )
    .unwrap();
    let above = Circle2D::new(Point2D::new(0.0, 1.5), 1.0).unwrap();
    // 円弧の母円とは下半分（y = -0.75）で交わるが、円弧とは交わらない
    let below = Circle2D::new(Point2D::new(0.0, -1.5), 1.0).unwrap();
    let inner = Circle2D::new(Point2D::new(0.0, 0.0), 0.5).unwrap();
    let enclosing = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();
    let tangent = Circle2D::new(Point2D::new(0.0, 3.0), 2.0).unwrap();

    assert!(upper_arc.collides(&above, TOLERANCE));
    assert!(above.collides(&upper_arc, TOLERANCE));
    assert!(upper_arc.collides(&tangent, TOLERANCE));
    assert!(!upper_arc.collides(&below, TOLERANCE));
    assert!(!upper_arc.collides(&inner, TOLERANCE));
    assert!(!upper_arc.collides(&enclosing, TOLERANCE));
}

#[test]
fn collision_2d_triangle_and_circle_treat_the_circle_as_a_curve() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(4.0, 0.0),
        Point2D::new(0.0, 4.0),
    )
    .unwrap();
    let crossing = Circle2D::new(Point2D::new(1.0, 1.0), 1.5).unwrap();
    let touching = Circle2D::new(Point2D::new(1.0, 1.0), 1.0).unwrap();
    let inside = Circle2D::new(Point2D::new(1.0, 1.0), 0.2).unwrap();
    let enclosing = Circle2D::new(Point2D::new(1.0, 1.0), 10.0).unwrap();
    let apart = Circle2D::new(Point2D::new(10.0, 10.0), 1.0).unwrap();

    assert!(triangle.collides(&crossing, TOLERANCE));
    assert!(crossing.collides(&triangle, TOLERANCE));
    assert!(triangle.collides(&touching, TOLERANCE));
    // 三角形は領域なので、内部にある円とは交わる
    assert!(triangle.collides(&inside, TOLERANCE));
    assert!(!triangle.collides(&enclosing, TOLERANCE));
    assert!(!triangle.collides(&apart, TOLERANCE));
}

fn ellipse() -> Ellipse2D<f64> {
    Ellipse2D::new(Point2D::new(0.0, 0.0), 2.0, 1.0, Angle::from_degrees(0.0)).unwrap()
}

fn ellipse_arc(start_degrees: f64, end_degrees: f64) -> EllipseArc2D<f64> {
    EllipseArc2D::new(
        Point2D::new(0.0, 0.0),
        2.0,
        1.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(start_degrees),
        Angle::from_degrees(end_degrees),
    )
    .unwrap()
}

#[test]
fn collision_2d_ellipse_and_circle_treat_both_as_curves() {
    let ellipse = ellipse();
    let crossing = Circle2D::new(Point2D::new(2.0, 0.0), 0.5).unwrap();
    let tangent = Circle2D::new(Point2D::new(0.0, 2.0), 1.0).unwrap();
    let inside = Circle2D::new(Point2D::new(0.0, 0.0), 0.5).unwrap();
    let enclosing = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

    assert!(ellipse.collides(&crossing, TOLERANCE));
    assert!(crossing.collides(&ellipse, TOLERANCE));
    assert!(ellipse.collides(&tangent, TOLERANCE));
    assert!(!ellipse.collides(&inside, TOLERANCE));
    assert!(!ellipse.collides(&enclosing, TOLERANCE));

    // 90 度回転した楕円は長軸が y 軸に沿い、長軸の端が (0, 2) にある
    let rotated =
        Ellipse2D::new(Point2D::new(0.0, 0.0), 2.0, 1.0, Angle::from_degrees(90.0)).unwrap();
    let at_rotated_vertex = Circle2D::new(Point2D::new(0.0, 2.5), 0.5).unwrap();
    assert!(rotated.collides(&at_rotated_vertex, TOLERANCE));
    assert!(!ellipse.collides(&at_rotated_vertex, TOLERANCE));
    assert_eq!(
        point_count(&at_rotated_vertex.intersection(&rotated, TOLERANCE)),
        1
    );
}

#[test]
fn collision_2d_ellipse_arc_and_circle_follow_the_arc_range() {
    let upper = ellipse_arc(0.0, 180.0);
    let near_top = Circle2D::new(Point2D::new(0.0, 1.0), 0.2).unwrap();
    // 母楕円とは下側の短軸の端の近くで交わるが、上半分の楕円弧とは交わらない
    let near_bottom = Circle2D::new(Point2D::new(0.0, -1.0), 0.2).unwrap();
    assert!(upper.collides(&near_top, TOLERANCE));
    assert!(near_top.collides(&upper, TOLERANCE));
    assert!(!upper.collides(&near_bottom, TOLERANCE));

    // 楕円全体の最近点（上側の短軸の端）は範囲外だが、範囲内の極小点（下側の短軸の端）で接する
    let lower = ellipse_arc(180.0, 360.0);
    let touching_bottom = Circle2D::new(Point2D::new(0.0, 0.5), 1.5).unwrap();
    let short_of_bottom = Circle2D::new(Point2D::new(0.0, 0.5), 1.4).unwrap();
    assert!(lower.collides(&touching_bottom, TOLERANCE));
    assert!(!lower.collides(&short_of_bottom, TOLERANCE));
}

#[test]
fn intersection_2d_circle_and_ellipse() {
    use geo_algorithms::IntersectionTopology;

    let ellipse = ellipse();
    let crossing = Circle2D::new(Point2D::new(0.0, 0.0), 1.5).unwrap();
    let result = crossing.intersection(&ellipse, TOLERANCE);
    assert_eq!(result.topology, IntersectionTopology::Crossing);
    assert_eq!(point_count(&result), 4);
    assert_eq!(point_count(&ellipse.intersection(&crossing, TOLERANCE)), 4);

    let off_center = Circle2D::new(Point2D::new(1.0, 0.5), 1.2).unwrap();
    assert_eq!(
        point_count(&off_center.intersection(&ellipse, TOLERANCE)),
        2
    );

    for tangent in [
        Circle2D::new(Point2D::new(0.0, 2.0), 1.0).unwrap(),
        Circle2D::new(Point2D::new(3.0, 0.0), 1.0).unwrap(),
    ] {
        let result = tangent.intersection(&ellipse, TOLERANCE);
        assert_eq!(result.topology, IntersectionTopology::Touching);
        assert_eq!(point_count(&result), 1);
    }

    let enclosing = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();
    assert!(!enclosing.intersection(&ellipse, TOLERANCE).intersects());

    let circular =
        Ellipse2D::new(Point2D::new(0.0, 0.0), 1.0, 1.0, Angle::from_degrees(0.0)).unwrap();
    let same = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
    assert_eq!(
        same.intersection(&circular, TOLERANCE).topology,
        IntersectionTopology::Coincident
    );
}

#[test]
fn intersection_2d_triangle_clips_the_segment() {
    use geo_algorithms::{IntersectionGeometry, IntersectionTopology};
    use geo_contracts::LineSegment2DProperties;

    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(4.0, 0.0),
        Point2D::new(0.0, 4.0),
    )
    .unwrap();
    let overlap_of = |segment: &LineSegment2D<f64>| match triangle.intersection(segment, TOLERANCE)
    {
        IntersectionResult {
            geometry: IntersectionGeometry::Segment2D(overlap),
            ..
        } => Some((overlap.start_point(), overlap.end_point())),
        _ => None,
    };

    let inside = LineSegment2D::new(Point2D::new(0.5, 0.5), Point2D::new(1.5, 1.0)).unwrap();
    let (start, end) = overlap_of(&inside).unwrap();
    assert!(start.distance_to(Point2D::new(0.5, 0.5)) < TOLERANCE);
    assert!(end.distance_to(Point2D::new(1.5, 1.0)) < TOLERANCE);
    assert!(triangle.collides(&inside, TOLERANCE));
    assert!(inside.collides(&triangle, TOLERANCE));

    let crossing = LineSegment2D::new(Point2D::new(-1.0, 1.0), Point2D::new(5.0, 1.0)).unwrap();
    let (start, end) = overlap_of(&crossing).unwrap();
    assert!(start.distance_to(Point2D::new(0.0, 1.0)) < TOLERANCE);
    assert!(end.distance_to(Point2D::new(3.0, 1.0)) < TOLERANCE);
    assert!(crossing.intersection(&triangle, TOLERANCE).intersects());

    let through_vertex =
        LineSegment2D::new(Point2D::new(4.0, -1.0), Point2D::new(4.0, 1.0)).unwrap();
    let touching = triangle.intersection(&through_vertex, TOLERANCE);
    assert_eq!(touching.topology, IntersectionTopology::Touching);
    assert_eq!(point_count(&touching), 1);

    let outside = LineSegment2D::new(Point2D::new(3.0, 3.0), Point2D::new(5.0, 5.0)).unwrap();
    assert!(!triangle.intersection(&outside, TOLERANCE).intersects());
    assert!(!triangle.collides(&outside, TOLERANCE));
}

#[test]
fn intersection_2d_returns_points_in_both_directions() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
    let ray = Ray2D::new(Point2D::new(-2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
    assert_eq!(point_count(&ray.intersection(&circle, TOLERANCE)), 2);
    assert_eq!(point_count(&circle.intersection(&ray, TOLERANCE)), 2);

    let inside_ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
    assert_eq!(point_count(&inside_ray.intersection(&circle, TOLERANCE)), 1);

    let line = InfiniteLine2D::new(Point2D::new(0.0, 0.0), Vector2D::new(0.0, 1.0)).unwrap();
    assert_eq!(point_count(&line.intersection(&circle, TOLERANCE)), 2);

    let segment = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();
    assert!(ray.intersection(&segment, TOLERANCE).intersects());
    assert!(segment.intersection(&ray, TOLERANCE).intersects());
    assert!(!line.intersection(&segment, TOLERANCE).intersects());
    assert!(line.intersection(&ray, TOLERANCE).intersects());
    assert!(ray.intersection(&line, TOLERANCE).intersects());
}

#[test]
fn collision_3d_follows_intersection() {
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
    let crossing =
        LineSegment3D::new(Point3D::new(0.0, -1.0, 0.0), Point3D::new(0.0, 1.0, 0.0)).unwrap();
    let short =
        LineSegment3D::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(0.0, 2.0, 0.0)).unwrap();
    assert!(line.collides(&crossing, TOLERANCE));
    assert!(crossing.collides(&line, TOLERANCE));
    assert!(!line.collides(&short, TOLERANCE));

    let toward = Ray3D::new(Point3D::new(0.0, 2.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();
    let away = Ray3D::new(Point3D::new(0.0, 2.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
    assert!(line.collides(&toward, TOLERANCE));
    assert!(toward.collides(&line, TOLERANCE));
    assert!(!line.collides(&away, TOLERANCE));
}

#[test]
fn distance_3d_handles_skew_parallel_and_aabb() {
    let x_axis = InfiniteLine3D::from_two_points(
        Point3D::new(0.0_f64, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    let skew =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, 2.0), Point3D::new(0.0, 1.0, 2.0))
            .unwrap();
    let parallel =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 3.0, 4.0), Point3D::new(1.0, 3.0, 4.0))
            .unwrap();
    let crossing =
        InfiniteLine3D::from_two_points(Point3D::new(5.0, -1.0, 0.0), Point3D::new(5.0, 1.0, 0.0))
            .unwrap();
    assert!((x_axis.distance_to(&skew) - 2.0).abs() < TOLERANCE);
    assert!((x_axis.distance_to(&parallel) - 5.0).abs() < TOLERANCE);
    assert!(x_axis.distance_to(&crossing) < TOLERANCE);

    let aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0));
    let through =
        LineSegment3D::new(Point3D::new(-1.0, 0.5, 0.5), Point3D::new(2.0, 0.5, 0.5)).unwrap();
    let above =
        LineSegment3D::new(Point3D::new(0.0, 0.5, 3.0), Point3D::new(1.0, 0.5, 3.0)).unwrap();
    assert!(through.distance_to(&aabb) < TOLERANCE);
    assert!((above.distance_to(&aabb) - 2.0).abs() < TOLERANCE);
}

#[test]
fn intersection_3d_linear_and_planar() {
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
    let other_line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, -1.0, 0.0), Point3D::new(0.0, 1.0, 0.0))
            .unwrap();
    assert!(line.intersection(&other_line, TOLERANCE).intersects());

    let segment_a =
        LineSegment3D::new(Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
    let segment_b =
        LineSegment3D::new(Point3D::new(0.0, -1.0, 0.0), Point3D::new(0.0, 1.0, 0.0)).unwrap();
    let segment_apart =
        LineSegment3D::new(Point3D::new(0.0, -1.0, 1.0), Point3D::new(0.0, 1.0, 1.0)).unwrap();
    assert!(segment_a.intersection(&segment_b, TOLERANCE).intersects());
    assert!(!segment_a
        .intersection(&segment_apart, TOLERANCE)
        .intersects());
    assert!(line.intersection(&segment_b, TOLERANCE).intersects());

    let ray_a = Ray3D::new(Point3D::new(-1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray_b = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
    let ray_away = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();
    assert!(ray_a.intersection(&ray_b, TOLERANCE).intersects());
    assert!(!ray_a.intersection(&ray_away, TOLERANCE).intersects());
    assert!(other_line.intersection(&ray_a, TOLERANCE).intersects());
    assert!(ray_a.intersection(&other_line, TOLERANCE).intersects());

    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.0), Direction3D::positive_z())
            .unwrap();
    let vertical =
        LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();
    let up = Ray3D::new(Point3D::new(0.0, 0.0, -1.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let down = Ray3D::new(Point3D::new(0.0, 0.0, -1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();
    assert!(plane.intersection(&vertical, TOLERANCE).intersects());
    assert!(plane.intersection(&up, TOLERANCE).intersects());
    assert!(!plane.intersection(&down, TOLERANCE).intersects());

    let triangle = Triangle3D::new(
        Point3D::new(-1.0, -1.0, 0.0),
        Point3D::new(1.0, -1.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let outside =
        LineSegment3D::new(Point3D::new(5.0, 5.0, -1.0), Point3D::new(5.0, 5.0, 1.0)).unwrap();
    assert!(triangle.intersection(&vertical, TOLERANCE).intersects());
    assert!(vertical.intersection(&triangle, TOLERANCE).intersects());
    assert!(!triangle.intersection(&outside, TOLERANCE).intersects());
    assert!(triangle.intersection(&up, TOLERANCE).intersects());
    assert!(up.intersection(&triangle, TOLERANCE).intersects());
    assert!(!triangle.intersection(&down, TOLERANCE).intersects());
}

#[test]
fn intersection_3d_quadric() {
    let sphere = SphericalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
    let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let away = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(-1.0, 0.0, 0.0)).unwrap();
    assert_eq!(point_count(&sphere.intersection(&ray, TOLERANCE)), 2);
    assert_eq!(point_count(&ray.intersection(&sphere, TOLERANCE)), 2);
    assert!(!sphere.intersection(&away, TOLERANCE).intersects());

    let chord =
        LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(point_count(&chord.intersection(&sphere, TOLERANCE)), 1);

    let solid = SphericalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
    assert!(solid
        .intersection(&Point3D::new(0.5, 0.0, 0.0), TOLERANCE)
        .intersects());
    assert!(!solid
        .intersection(&Point3D::new(2.0, 0.0, 0.0), TOLERANCE)
        .intersects());

    let cone_solid = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0, 2.0).unwrap();
    assert!(cone_solid
        .intersection(&Point3D::new(0.0, 0.0, 0.5), TOLERANCE)
        .intersects());
    assert!(!cone_solid
        .intersection(&Point3D::new(5.0, 0.0, 0.0), TOLERANCE)
        .intersects());

    let cone =
        ConicalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0, Angle::from_degrees(45.0))
            .unwrap();
    assert!(cone
        .intersection(&Point3D::new(1.0, 0.0, 0.0), TOLERANCE)
        .intersects());
    assert!(!cone
        .intersection(&Point3D::new(0.0, 0.0, 0.0), TOLERANCE)
        .intersects());

    let horizontal_line =
        InfiniteLine3D::from_two_points(Point3D::new(-3.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0))
            .unwrap();
    assert_eq!(
        point_count(&cone.intersection(&horizontal_line, TOLERANCE)),
        2
    );
    let horizontal_ray =
        Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        point_count(&cone.intersection(&horizontal_ray, TOLERANCE)),
        1
    );
    assert_eq!(
        point_count(&horizontal_ray.intersection(&cone, TOLERANCE)),
        1
    );
    let horizontal_segment =
        LineSegment3D::new(Point3D::new(-3.0, 0.0, 0.0), Point3D::new(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        point_count(&cone.intersection(&horizontal_segment, TOLERANCE)),
        1
    );
}

/// 交差の結果が線分なら両端の点を返す
fn segment_ends(result: &IntersectionResult<f64>) -> Option<(Point3D<f64>, Point3D<f64>)> {
    use geo_algorithms::IntersectionGeometry;
    use geo_contracts::LineSegment3DProperties;
    match &result.geometry {
        IntersectionGeometry::Segment(segment) => {
            Some((segment.start_point(), segment.end_point()))
        }
        _ => None,
    }
}

fn assert_segment(result: &IntersectionResult<f64>, start: Point3D<f64>, end: Point3D<f64>) {
    let (actual_start, actual_end) = segment_ends(result).expect("expected a segment");
    assert!(
        actual_start.distance_to(start) < 1e-9,
        "start: {actual_start:?}"
    );
    assert!(actual_end.distance_to(end) < 1e-9, "end: {actual_end:?}");
}

fn line_along_x(y: f64, z: f64) -> InfiniteLine3D<f64> {
    InfiniteLine3D::from_two_points(Point3D::new(-3.0, y, z), Point3D::new(3.0, y, z)).unwrap()
}

#[test]
fn intersection_3d_spherical_solid_returns_the_chord() {
    use geo_algorithms::IntersectionTopology;

    let sphere = SphericalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
    assert_segment(
        &sphere.intersection(&line_along_x(0.0, 0.0), TOLERANCE),
        Point3D::new(-1.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    );

    let tangent = sphere.intersection(&line_along_x(1.0, 0.0), TOLERANCE);
    assert_eq!(tangent.topology, IntersectionTopology::Touching);
    let near_tangent = sphere.intersection(&line_along_x(1.0 + TOLERANCE / 2.0, 0.0), TOLERANCE);
    assert_eq!(near_tangent.topology, IntersectionTopology::Touching);
    assert!(!sphere
        .intersection(&line_along_x(1.1, 0.0), TOLERANCE)
        .intersects());

    let from_center =
        Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert_segment(
        &sphere.intersection(&from_center, TOLERANCE),
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    );
    let toward = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(-1.0, 0.0, 0.0)).unwrap();
    assert_segment(
        &sphere.intersection(&toward, TOLERANCE),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(-1.0, 0.0, 0.0),
    );
    let away = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert!(!sphere.intersection(&away, TOLERANCE).intersects());
}

#[test]
fn intersection_3d_conical_solid_returns_the_inside_interval() {
    use geo_algorithms::IntersectionTopology;

    // 底面の中心が原点・半径 1・高さ 2（頂点は (0, 0, 2)）。高さ 1 の断面の半径は 0.5
    let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0, 2.0).unwrap();
    assert_segment(
        &cone.intersection(&line_along_x(0.0, 1.0), TOLERANCE),
        Point3D::new(-0.5, 0.0, 1.0),
        Point3D::new(0.5, 0.0, 1.0),
    );
    let along_axis =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 5.0))
            .unwrap();
    assert_segment(
        &cone.intersection(&along_axis, TOLERANCE),
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(0.0, 0.0, 2.0),
    );
    // 底面上の直線は底面の円板と重なる
    assert_segment(
        &cone.intersection(&line_along_x(0.0, 0.0), TOLERANCE),
        Point3D::new(-1.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    );

    let tangent = cone.intersection(&line_along_x(0.5, 1.0), TOLERANCE);
    assert_eq!(tangent.topology, IntersectionTopology::Touching);
    assert_eq!(point_count(&tangent), 1);
    let near_tangent = cone.intersection(&line_along_x(0.5 + TOLERANCE / 2.0, 1.0), TOLERANCE);
    assert_eq!(near_tangent.topology, IntersectionTopology::Touching);
    assert!(!cone
        .intersection(&line_along_x(0.6, 1.0), TOLERANCE)
        .intersects());
    assert!(!cone
        .intersection(&line_along_x(0.0, -0.5), TOLERANCE)
        .intersects());
    assert!(!cone
        .intersection(&line_along_x(0.0, 2.5), TOLERANCE)
        .intersects());

    let from_axis = Ray3D::new(Point3D::new(0.0, 0.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert_segment(
        &cone.intersection(&from_axis, TOLERANCE),
        Point3D::new(0.0, 0.0, 1.0),
        Point3D::new(0.5, 0.0, 1.0),
    );
    let away = Ray3D::new(Point3D::new(2.0, 0.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert!(!cone.intersection(&away, TOLERANCE).intersects());

    let half =
        LineSegment3D::new(Point3D::new(-2.0, 0.0, 1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();
    assert_segment(
        &cone.intersection(&half, TOLERANCE),
        Point3D::new(-0.5, 0.0, 1.0),
        Point3D::new(0.0, 0.0, 1.0),
    );
    let short =
        LineSegment3D::new(Point3D::new(-2.0, 0.0, 1.0), Point3D::new(-1.0, 0.0, 1.0)).unwrap();
    assert!(!cone.intersection(&short, TOLERANCE).intersects());

    // 軸が x 軸正方向の円錐（底面の中心 (1, 0, 0)、頂点 (3, 0, 0)）
    let tilted = ConicalSolid3D::new(
        Point3D::new(1.0, 0.0, 0.0),
        Direction3D::positive_x(),
        Direction3D::positive_z(),
        1.0,
        2.0,
    )
    .unwrap();
    assert_segment(
        &tilted.intersection(&line_along_x(0.0, 0.0), TOLERANCE),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(3.0, 0.0, 0.0),
    );
    let across =
        InfiniteLine3D::from_two_points(Point3D::new(2.0, -3.0, 0.0), Point3D::new(2.0, 3.0, 0.0))
            .unwrap();
    assert_segment(
        &tilted.intersection(&across, TOLERANCE),
        Point3D::new(2.0, -0.5, 0.0),
        Point3D::new(2.0, 0.5, 0.0),
    );
}

/// 中心が原点・半径 1 の xy 平面上の円（次数 2 の有理曲線、4 つの 4 分円）
fn nurbs_unit_circle() -> NurbsCurve3D<f64> {
    use geo_contracts::NurbsCurve3DConstructor;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let points = [
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (-1.0, 1.0),
        (-1.0, 0.0),
        (-1.0, -1.0),
        (0.0, -1.0),
        (1.0, -1.0),
        (1.0, 0.0),
    ]
    .map(|(x, y)| Point3D::new(x, y, 0.0));
    NurbsCurve3D::new(
        2,
        vec![
            0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
        ],
        &points,
        Some(vec![1.0, s, 1.0, s, 1.0, s, 1.0, s, 1.0]),
    )
    .unwrap()
}

#[test]
fn distance_3d_nurbs_curve_to_shapes() {
    use geo_contracts::{
        default_distance_tolerance, Circle3DConstructor, CylindricalSolid3DConstructor,
        EllipsoidalSolid3DConstructor,
    };

    let tolerance = default_distance_tolerance::<f64>();
    let assert_distance = |actual: f64, expected: f64| {
        // 曲線上の点での距離を返すので、真の値以上・真の値 + 許容誤差以下
        assert!(
            expected - 1e-12 <= actual && actual <= expected + tolerance,
            "distance: {actual}, expected: {expected}"
        );
    };
    let curve = nurbs_unit_circle();

    let circle =
        Circle3D::new(Point3D::new(4.0, 0.0, 0.0), Direction3D::positive_z(), 1.0).unwrap();
    assert_distance(curve.distance_to(&circle), 2.0);
    assert_distance(circle.distance_to(&curve), 2.0);

    // 軸が z 軸方向で (3, 0, -1) から高さ 2 の円柱
    let cylinder =
        CylindricalSolid3D::new_standard(Point3D::new(3.0, 0.0, -1.0), 1.0, 2.0).unwrap();
    assert_distance(curve.distance_to(&cylinder), 1.0);
    // x 方向の半径 1 の楕円体。曲線上の (1, 0, 0) と楕円体の (2, 0, 0) が最も近い
    let ellipsoid =
        EllipsoidalSolid3D::new_standard(Point3D::new(3.0, 0.0, 0.0), 1.0, 2.0, 2.0).unwrap();
    assert_distance(curve.distance_to(&ellipsoid), 1.0);

    let vertical =
        InfiniteLine3D::from_two_points(Point3D::new(3.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 1.0))
            .unwrap();
    assert_distance(curve.distance_to(&vertical), 2.0);
    let segment =
        LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(5.0, 0.0, 0.0)).unwrap();
    assert_distance(curve.distance_to(&segment), 1.0);
    let crossing =
        LineSegment3D::new(Point3D::new(0.0, -2.0, 0.0), Point3D::new(0.0, 2.0, 0.0)).unwrap();
    assert_distance(curve.distance_to(&crossing), 0.0);
    let ray = Ray3D::new(Point3D::new(3.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert_distance(curve.distance_to(&ray), 2.0);

    let above =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 2.0), Direction3D::positive_z())
            .unwrap();
    assert_distance(curve.distance_to(&above), 2.0);
    assert_distance(above.distance_to(&curve), 2.0);
    let through =
        Plane3D::from_point_and_normal(Point3D::new(0.5, 0.0, 0.0), Direction3D::positive_x())
            .unwrap();
    assert_distance(curve.distance_to(&through), 0.0);

    let sphere = SphericalSolid3D::new_standard(Point3D::new(3.0, 0.0, 0.0), 1.0).unwrap();
    assert_distance(curve.distance_to(&sphere), 1.0);
    let inside = SphericalSolid3D::new_standard(Point3D::new(1.0, 0.0, 0.0), 0.5).unwrap();
    assert_distance(curve.distance_to(&inside), 0.0);
}
