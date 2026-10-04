//! 円・楕円系形状の相似変換テスト

use crate::{
    Arc2D, Arc3D, Circle2D, Circle3D, Direction3D, Ellipse2D, Ellipse3D, EllipseArc2D,
    EllipseArc3D, Point2D, Point3D, SimilarityTransform2D, SimilarityTransform3D, Vector2D,
    Vector3D,
};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    Angle, Arc2DEvaluation, Arc3DEndpoint, Arc3DEvaluation, Circle3DEvaluation,
    SimilarityTransformable2D, SimilarityTransformable3D,
};
use std::f64::consts::PI;

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

fn map3(p: (f64, f64, f64)) -> Point3D<f64> {
    Point3D::from(p)
        .transform_similarity(&transform_3d())
        .unwrap()
}

fn map2(p: Point2D<f64>) -> Point2D<f64> {
    p.transform_similarity(&transform_2d()).unwrap()
}

fn assert_point3_eq(actual: Point3D<f64>, expected: Point3D<f64>) {
    assert!(
        actual.distance_to(&expected) < TOLERANCE_F64,
        "{actual:?} != {expected:?}"
    );
}

fn assert_point2_eq(actual: Point2D<f64>, expected: Point2D<f64>) {
    assert!(
        actual.distance_to(&expected) < TOLERANCE_F64,
        "{actual:?} != {expected:?}"
    );
}

const PARAMS: [f64; 4] = [0.0, 0.3, 1.7, 4.0];

#[test]
fn circle_3d_maps_every_parameter_and_scales_radius() {
    let circle = Circle3D::new_xy_plane(Point3D::new(1.0, 0.0, 0.0), 2.0).unwrap();
    let moved = circle.transform_similarity(&transform_3d()).unwrap();

    assert!((moved.radius_internal() - 4.0).abs() < TOLERANCE_F64);
    for t in PARAMS {
        let expected = map3(Circle3DEvaluation::point_at_parameter(&circle, t));
        let actual = Point3D::from(Circle3DEvaluation::point_at_parameter(&moved, t));
        assert_point3_eq(actual, expected);
    }
}

#[test]
fn arc_3d_keeps_angles_and_maps_endpoints() {
    let arc = Arc3D::new(
        Point3D::new(1.0, 0.0, 0.0),
        2.0,
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        Angle::from_degrees(30.0),
        Angle::from_degrees(150.0),
    )
    .unwrap();
    let moved = arc.transform_similarity(&transform_3d()).unwrap();

    assert_point3_eq(
        Point3D::from(Arc3DEndpoint::start_point(&moved)),
        map3(Arc3DEndpoint::start_point(&arc)),
    );
    assert_point3_eq(
        Point3D::from(Arc3DEndpoint::end_point(&moved)),
        map3(Arc3DEndpoint::end_point(&arc)),
    );
    for t in [0.25, 0.5, 0.75] {
        let expected = map3(Arc3DEvaluation::point_at_parameter(&arc, t));
        let actual = Point3D::from(Arc3DEvaluation::point_at_parameter(&moved, t));
        assert_point3_eq(actual, expected);
    }
}

#[test]
fn ellipse_3d_maps_every_parameter_and_scales_axes() {
    let ellipse = Ellipse3D::new(
        Point3D::new(1.0, 0.0, 0.0),
        3.0,
        1.0,
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    let moved = ellipse.transform_similarity(&transform_3d()).unwrap();

    assert!((moved.semi_major_axis() - 6.0).abs() < TOLERANCE_F64);
    assert!((moved.semi_minor_axis() - 2.0).abs() < TOLERANCE_F64);
    for t in PARAMS {
        let expected = ellipse
            .point_at_parameter(t)
            .transform_similarity(&transform_3d())
            .unwrap();
        assert_point3_eq(moved.point_at_parameter(t), expected);
    }
}

#[test]
fn ellipse_arc_3d_keeps_angles_and_maps_points() {
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        3.0,
        1.0,
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    let arc = EllipseArc3D::new(
        ellipse,
        Angle::from_degrees(10.0),
        Angle::from_degrees(200.0),
    );
    let moved = arc.transform_similarity(&transform_3d()).unwrap();

    for t in [0.0, 0.5, 1.0] {
        let expected = arc
            .point_at_parameter(t)
            .transform_similarity(&transform_3d())
            .unwrap();
        assert_point3_eq(moved.point_at_parameter(t), expected);
    }
}

#[test]
fn circle_2d_scales_radius_and_keeps_points_on_circle() {
    let circle = Circle2D::new(Point2D::new(1.0, 0.0), 2.0).unwrap();
    let moved = circle.transform_similarity(&transform_2d()).unwrap();

    assert_point2_eq(moved.center_internal(), map2(circle.center_internal()));
    assert!((moved.radius_internal() - 4.0).abs() < TOLERANCE_F64);
    // 2D の円のパラメータはグローバル X 軸基準のため、点の集合として一致を確認する
    for t in PARAMS {
        let mapped = map2(circle.point_at_parameter(t));
        let distance = mapped.distance_to(&moved.center_internal());
        assert!((distance - 4.0).abs() < TOLERANCE_F64);
    }
}

#[test]
fn arc_2d_adds_rotation_to_angles_and_maps_endpoints() {
    let arc = Arc2D::from_center_radius(
        Point2D::new(1.0, 0.0),
        2.0,
        Angle::from_degrees(300.0),
        Angle::from_degrees(400.0),
    )
    .unwrap();
    let moved = arc.transform_similarity(&transform_2d()).unwrap();

    assert_point2_eq(moved.start_point(), map2(arc.start_point()));
    assert_point2_eq(moved.end_point(), map2(arc.end_point()));
    for t in [0.25, 0.5, 0.75] {
        let (x, y) = Arc2DEvaluation::point_at_parameter(&arc, t);
        let expected = map2(Point2D::new(x, y));
        let (x, y) = Arc2DEvaluation::point_at_parameter(&moved, t);
        let actual = Point2D::new(x, y);
        assert_point2_eq(actual, expected);
    }
    // 開始角は [0, 2π) に正規化し、掃引角は保つ
    let start = moved.start_angle.to_radians();
    assert!((0.0..2.0 * PI).contains(&start));
    let sweep = (moved.end_angle - moved.start_angle).to_radians();
    assert!((sweep - (arc.end_angle - arc.start_angle).to_radians()).abs() < TOLERANCE_F64);
}

#[test]
fn ellipse_2d_updates_rotation_and_maps_every_parameter() {
    let ellipse = Ellipse2D::new(Point2D::new(1.0, 0.0), 3.0, 1.0, PI / 6.0).unwrap();
    let moved = ellipse.transform_similarity(&transform_2d()).unwrap();

    for t in PARAMS {
        assert_point2_eq(
            moved.point_at_parameter(t),
            map2(ellipse.point_at_parameter(t)),
        );
    }
}

#[test]
fn ellipse_arc_2d_keeps_angles_and_maps_points() {
    let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 3.0, 1.0, 0.0).unwrap();
    let arc = EllipseArc2D::new(
        ellipse,
        Angle::from_degrees(10.0),
        Angle::from_degrees(200.0),
    );
    let moved = arc.transform_similarity(&transform_2d()).unwrap();

    for t in [0.0, 0.5, 1.0] {
        assert_point2_eq(moved.point_at_parameter(t), map2(arc.point_at_parameter(t)));
    }
}
