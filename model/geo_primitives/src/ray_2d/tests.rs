//! Ray2D のテスト

use crate::{Point2D, Ray2D, Vector2D};
use geo_contracts::{
    Ray2DConstructor, Ray2DContainment, Ray2DDistance, Ray2DEvaluation, Ray2DProjection,
};

#[test]
fn test_distance_to_point() {
    let ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
    let tolerance = 1e-3;

    assert!(ray.distance_to_point(Point2D::new(5.0, 0.0)) <= tolerance);
    assert!(ray.distance_to_point(Point2D::new(5.0, 0.5 * tolerance)) <= tolerance);
    assert!(ray.distance_to_point(Point2D::new(5.0, 2.0 * tolerance)) > tolerance);
    assert!(ray.distance_to_point(Point2D::new(-2.0, 0.0)) > tolerance);

    // 起点の後方は、起点までの距離を返す（Ray3D と同じ）
    assert!(ray.distance_to_point(Point2D::new(-0.5 * tolerance, 0.0)) <= tolerance);
    assert!(ray.distance_to_point(Point2D::new(-2.0 * tolerance, 0.0)) > tolerance);
}

#[test]
fn test_contains_point_uses_default_tolerance() {
    let ray = Ray2D::new(Point2D::new(0.0_f64, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

    assert!(ray.contains_point(Point2D::new(5.0, 0.0)));
    assert!(!ray.contains_point(Point2D::new(5.0, 1.0)));
    assert!(!ray.contains_point(Point2D::new(-1.0, 0.0)));
}

#[test]
fn test_closest_point() {
    let ray = Ray2D::new(Point2D::new(1.0_f64, 1.0), Vector2D::new(0.0, 1.0)).unwrap();

    // 起点の前方は投影した位置、後方は起点
    assert!(
        ray.closest_point(Point2D::new(3.0, 4.0))
            .distance_to(Point2D::new(1.0, 4.0))
            < 1e-12
    );
    assert!(
        ray.closest_point(Point2D::new(3.0, -2.0))
            .distance_to(Point2D::new(1.0, 1.0))
            < 1e-12
    );

    // parameter_for_point は起点の後方で負の値を返す
    assert!((ray.parameter_for_point(Point2D::new(3.0, -2.0)) + 3.0).abs() < 1e-12);
}
