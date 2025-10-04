/// 3Dプリミティブのユニットテスト

use crate::primitives3d::{Point3D, LineSegment3D, Plane, Sphere, ParametricCurve3D, Circle3D};
use crate::{scalar::Scalar, vector::Direction3D, tolerance::ToleranceContext};

#[test]
fn test_point_distance_3d() {
    let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
    let p2 = Point3D::from_f64(1.0, 1.0, 1.0);
    let distance = p1.distance_to(&p2);
    assert!((distance.value() - f64::sqrt(3.0)).abs() < 1e-10);
}

#[test]
fn test_plane_from_three_points() {
    let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
    let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
    let p3 = Point3D::from_f64(0.0, 1.0, 0.0);

    let plane = Plane::from_three_points(&p1, &p2, &p3).unwrap();

    // Z軸方向が法線になるはず
    assert!((plane.normal().z() - 1.0).abs() < 1e-10);
}

#[test]
fn test_sphere_volume() {
    let center = Point3D::origin();
    let radius = Scalar::new(1.0);
    let sphere = Sphere::new(center, radius);

    let expected_volume = 4.0 * std::f64::consts::PI / 3.0;
    assert!((sphere.volume().value() - expected_volume).abs() < 1e-10);
}

#[test]
fn test_line_segment_distance_to_point() {
    let start = Point3D::from_f64(0.0, 0.0, 0.0);
    let end = Point3D::from_f64(1.0, 0.0, 0.0);
    let line = LineSegment3D::new(start, end);

    let point = Point3D::from_f64(0.5, 1.0, 0.0);
    let distance = line.distance_to_point(&point);
    assert!((distance.value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_circle3d_basic() {
    let center = Point3D::origin();
    let ctx = ToleranceContext::standard();
    let normal = Direction3D::new(0.0, 0.0, 1.0, &ctx).unwrap();
    let circle = Circle3D::new(center.clone(), 2.0, normal);
    // t=0 -> (r,0,0)
    let p0 = circle.evaluate(Scalar::new(0.0));
    assert!((p0.x().value() - 2.0).abs() < 1e-10);
    assert!(p0.y().value().abs() < 1e-10);
    // 半周 t=0.5 -> (-r,0,0)
    let p_half = circle.evaluate(Scalar::new(0.5));
    assert!((p_half.x().value() + 2.0).abs() < 1e-10);
    assert!(p_half.y().value().abs() < 1e-10);
    // 長さ
    let len = circle.length();
    assert!((len.value() - 2.0 * std::f64::consts::PI * 2.0).abs() < 1e-10);
    // 導関数 t=0 -> (0, 2πr, 0)
    let d0 = circle.derivative(Scalar::new(0.0));
    assert!(d0.x().abs() < 1e-10);
    assert!((d0.y() - (std::f64::consts::TAU * 2.0)).abs() < 1e-10);
}