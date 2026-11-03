//! 線形・平面系形状の相似変換テスト

use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    Angle, InfiniteLine2DConstructor, InfiniteLine2DEvaluation, InfiniteLine3DConstructor,
    InfiniteLine3DEvaluation, LineSegment2DConstructor, LineSegment2DProperties,
    LineSegment3DConstructor, LineSegment3DProperties, LineSegment3DSupportLine,
    Plane3DConstructor, Plane3DProperties, Ray2DConstructor, Ray2DEvaluation, Ray3DConstructor,
    Ray3DEvaluation, Ray3DProperties, Rect2DConstructor, Rect2DProperties, Rect3DConstructor,
    Rect3DProperties, SimilarityTransformable2D, SimilarityTransformable3D, TransformError,
    Triangle2DBoundaryAccess, Triangle2DConstructor, Triangle3DBoundaryAccess,
    Triangle3DConstructor, TriangleMesh3DConstructor, TriangleMesh3DProperties,
};
use geo_primitives::{
    Direction3D, InfiniteLine2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Plane3D, Point2D,
    Point3D, Ray2D, Ray3D, Rect2D, Rect3D, SimilarityTransform2D, SimilarityTransform3D,
    Triangle2D, Triangle3D, TriangleMesh3D, Vector2D, Vector3D,
};

/// z 軸まわりに 90° 回転 → (0, 0, 5) 平行移動 → 原点まわりに 2 倍
fn transform_3d() -> SimilarityTransform3D<f64> {
    SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Angle::from_degrees(90.0),
    )
    .then(&SimilarityTransform3D::translation(Vector3D::new(
        0.0, 0.0, 5.0,
    )))
    .then(&SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 2.0).unwrap())
}

/// 原点まわりに 90° 回転 → (0, 5) 平行移動 → 原点まわりに 2 倍
fn transform_2d() -> SimilarityTransform2D<f64> {
    SimilarityTransform2D::rotation_about(Point2D::new(0.0, 0.0), Angle::from_degrees(90.0))
        .then(&SimilarityTransform2D::translation(Vector2D::new(0.0, 5.0)))
        .then(&SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 2.0).unwrap())
}

fn assert_point3_eq(actual: Point3D<f64>, expected: Point3D<f64>) {
    assert!(
        actual.distance_to(expected) < TOLERANCE_F64,
        "{actual:?} != {expected:?}"
    );
}

fn assert_point2_eq(actual: Point2D<f64>, expected: Point2D<f64>) {
    assert!(
        actual.distance_to(expected) < TOLERANCE_F64,
        "{actual:?} != {expected:?}"
    );
}

fn map3(p: Point3D<f64>) -> Point3D<f64> {
    transform_3d().apply_point(p)
}

fn map2(p: Point2D<f64>) -> Point2D<f64> {
    transform_2d().apply_point(p)
}

#[test]
fn line_segment_3d_keeps_support_line_parameters_consistent() {
    let segment =
        LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
    let moved = segment.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(moved.start_point(), map3(segment.start_point()));
    assert_point3_eq(moved.end_point(), map3(segment.end_point()));
    assert!((moved.length() - 4.0).abs() < TOLERANCE_F64);
    // support line 上の理想端点と拘束端点が一致する
    assert_point3_eq(moved.constraint_start_point(), moved.start_point());
    assert_point3_eq(moved.constraint_end_point(), moved.end_point());
}

#[test]
fn ray_3d_transforms_origin_and_unit_direction() {
    let ray = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let moved = ray.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(moved.origin(), map3(ray.origin()));
    assert!(
        (moved.direction().as_vector() - Vector3D::new(0.0, 1.0, 0.0)).length() < TOLERANCE_F64
    );
    // パラメータは距離のため、スケール後は 2 倍の距離に対応する
    assert_point3_eq(
        moved.point_at_parameter(2.0),
        map3(ray.point_at_parameter(1.0)),
    );
}

#[test]
fn infinite_line_3d_transforms_points_on_line() {
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 1.0, 0.0), Point3D::new(1.0, 1.0, 0.0))
            .unwrap();
    let moved = line.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(
        moved.point_at_parameter(0.0),
        map3(line.point_at_parameter(0.0)),
    );
    assert_point3_eq(
        moved.point_at_parameter(2.0),
        map3(line.point_at_parameter(1.0)),
    );
}

#[test]
fn plane_3d_keeps_orthonormal_right_handed_frame() {
    let plane = Plane3D::from_origin_and_axes(
        Point3D::new(1.0, 0.0, 0.0),
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();
    let moved = plane.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(moved.origin(), map3(plane.origin()));
    assert!((moved.normal().as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < TOLERANCE_F64);
    let (n, u, v) = (
        moved.normal().as_vector(),
        Plane3DProperties::u_axis(&moved).as_vector(),
        Plane3DProperties::v_axis(&moved).as_vector(),
    );
    assert!((u - Vector3D::new(0.0, 1.0, 0.0)).length() < TOLERANCE_F64);
    assert!(u.dot(v).abs() < TOLERANCE_F64 && n.dot(u).abs() < TOLERANCE_F64);
    assert!((n.cross(u) - v).length() < TOLERANCE_F64);
}

#[test]
fn triangle_3d_transforms_vertices_and_scales_area() {
    let triangle = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let moved = triangle.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(moved.vertex_a(), map3(triangle.vertex_a()));
    assert_point3_eq(moved.vertex_c(), map3(triangle.vertex_c()));
    // 辺の長さはスケール係数倍
    let edge = |t: &Triangle3D<f64>| t.vertex_a().distance_to(t.vertex_b());
    assert!((edge(&moved) - edge(&triangle) * 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn triangle_mesh_3d_transforms_vertices_and_keeps_indices() {
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let moved = mesh.transform_similarity(&transform_3d()).unwrap();

    for (a, b) in moved.vertices().iter().zip(mesh.vertices()) {
        assert_point3_eq(*a, map3(*b));
    }
    assert_eq!(moved.indices(), mesh.indices());
}

#[test]
fn rect_3d_transforms_frame_and_scales_size() {
    let rect = Rect3D::new(
        Point3D::new(1.0, 0.0, 0.0),
        Direction3D::positive_x(),
        Direction3D::new(0.0, 1.0, 0.0).unwrap(),
        2.0,
        3.0,
    )
    .unwrap();
    let moved = rect.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(moved.origin(), map3(rect.origin()));
    assert!((moved.u_axis().as_vector() - Vector3D::new(0.0, 1.0, 0.0)).length() < TOLERANCE_F64);
    assert!((moved.width() - 4.0).abs() < TOLERANCE_F64);
    assert!((moved.height() - 6.0).abs() < TOLERANCE_F64);
}

#[test]
fn line_segment_2d_keeps_support_line_parameters_consistent() {
    let segment = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(3.0, 0.0)).unwrap();
    let moved = segment.transform_similarity(&transform_2d()).unwrap();

    assert_point2_eq(moved.start_point(), map2(segment.start_point()));
    assert_point2_eq(moved.end_point(), map2(segment.end_point()));
    assert!((moved.length() - 4.0).abs() < TOLERANCE_F64);
}

#[test]
fn ray_and_infinite_line_2d_transform_points() {
    let ray = Ray2D::new(Point2D::new(1.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
    let moved_ray = ray.transform_similarity(&transform_2d()).unwrap();
    assert_point2_eq(
        moved_ray.point_at_parameter(2.0),
        map2(ray.point_at_parameter(1.0)),
    );

    let line =
        InfiniteLine2D::from_two_points(Point2D::new(0.0, 1.0), Point2D::new(1.0, 1.0)).unwrap();
    let moved_line = line.transform_similarity(&transform_2d()).unwrap();
    assert_point2_eq(
        moved_line.point_at_parameter(2.0),
        map2(line.point_at_parameter(1.0)),
    );
}

#[test]
fn triangle_2d_transforms_vertices_and_scales_area() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(1.0, 0.0),
        Point2D::new(0.0, 1.0),
    )
    .unwrap();
    let moved = triangle.transform_similarity(&transform_2d()).unwrap();
    assert_point2_eq(moved.vertex_b(), map2(triangle.vertex_b()));
    // 辺の長さはスケール係数倍
    let edge = |t: &Triangle2D<f64>| t.vertex_a().distance_to(t.vertex_b());
    assert!((edge(&moved) - edge(&triangle) * 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn rect_2d_accepts_translation_and_scale_but_rejects_rotation() {
    let rect = Rect2D::new(Point2D::new(1.0, 1.0), 2.0, 3.0).unwrap();
    let transform = SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 2.0)
        .unwrap()
        .then(&SimilarityTransform2D::translation(Vector2D::new(1.0, 0.0)));
    let moved = rect.transform_similarity(&transform).unwrap();
    assert_point2_eq(moved.origin(), Point2D::new(3.0, 2.0));
    assert!((moved.width() - 4.0).abs() < TOLERANCE_F64);
    assert!((moved.height() - 6.0).abs() < TOLERANCE_F64);

    assert!(matches!(
        rect.transform_similarity(&transform_2d()),
        Err(TransformError::Unsupported(_))
    ));
}
