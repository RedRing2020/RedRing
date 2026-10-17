//! 形状をまたぐ演算の網羅表（`dev/architecture/CROSS_OPERATION_COVERAGE_MATRIX_SPEC.md`）
//!
//! 実装済みの組み合わせは trait の実装をコンパイル時に確かめ、代表的な配置で結果を確かめる。
//! 未対応の組み合わせは仕様書の未対応表で管理する。

use geo_algorithms::{
    Aabb3D, Angle, Arc2D, Circle2D, ConicalSolid3D, ConicalSurface3D, CrossCollision,
    CrossDistance, CrossIntersection, InfiniteLine2D, InfiniteLine3D, IntersectionResult,
    LineSegment2D, LineSegment3D, Plane3D, Point2D, Point3D, Ray2D, Ray3D, SphericalSolid3D,
    SphericalSurface3D, Triangle3D, Vector2D, Vector3D,
};
use geo_contracts::{
    Arc2DConstructor, Circle2DConstructor, ConicalSolid3DConstructor, ConicalSurface3DConstructor,
    InfiniteLine2DConstructor, InfiniteLine3DConstructor, LineSegment2DConstructor,
    LineSegment3DConstructor, Plane3DConstructor, Ray2DConstructor, Ray3DConstructor,
    SphericalSolid3DConstructor, SphericalSurface3DConstructor, Triangle3DConstructor,
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
    symmetric!(collision, InfiniteLine2D, Circle2D);
    symmetric!(collision, InfiniteLine2D, LineSegment2D);
    symmetric!(collision, InfiniteLine2D, Ray2D);

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
    one_way!(intersection, ConicalSurface3D, Point3D);
    one_way!(intersection, ConicalSurface3D, InfiniteLine3D);
    symmetric!(intersection, ConicalSurface3D, Ray3D);
    one_way!(intersection, ConicalSurface3D, LineSegment3D);
    one_way!(intersection, SphericalSolid3D, Point3D);
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

    let plane = Plane3D::from_point_and_normal(
        Point3D::new(0.0, 0.0, 0.0),
        geo_algorithms::Direction3D::positive_z(),
    )
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
