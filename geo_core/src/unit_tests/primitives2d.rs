/// 2Dプリミティブのユニットテスト

use crate::primitives2d::{Point2D, LineSegment2D, Arc2D, Polygon2D, ParametricCurve2D};
use crate::scalar::Scalar;

#[test]
fn test_point_distance() {
    let p1 = Point2D::from_f64(0.0, 0.0);
    let p2 = Point2D::from_f64(3.0, 4.0);
    let distance = p1.distance_to(&p2);
    assert!((distance.value() - 5.0).abs() < 1e-10);
}

#[test]
fn test_line_segment_evaluation() {
    let start = Point2D::from_f64(0.0, 0.0);
    let end = Point2D::from_f64(2.0, 2.0);
    let line = LineSegment2D::new(start, end);

    let mid_point = line.evaluate(Scalar::new(0.5));
    assert!((mid_point.x().value() - 1.0).abs() < 1e-10);
    assert!((mid_point.y().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_arc_evaluation() {
    let center = Point2D::from_f64(0.0, 0.0);
    let radius = Scalar::new(1.0);
    let arc = Arc2D::new(
        center,
        radius,
        Scalar::new(0.0),
        Scalar::new(std::f64::consts::PI / 2.0),
    );

    let start_point = arc.evaluate(Scalar::new(0.0));
    assert!((start_point.x().value() - 1.0).abs() < 1e-10);
    assert!((start_point.y().value() - 0.0).abs() < 1e-10);

    let end_point = arc.evaluate(Scalar::new(1.0));
    assert!((end_point.x().value() - 0.0).abs() < 1e-10);
    assert!((end_point.y().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_polygon_area() {
    // 単位正方形
    let vertices = vec![
        Point2D::from_f64(0.0, 0.0),
        Point2D::from_f64(1.0, 0.0),
        Point2D::from_f64(1.0, 1.0),
        Point2D::from_f64(0.0, 1.0),
    ];
    let polygon = Polygon2D::new(vertices);
    assert!((polygon.area().value() - 1.0).abs() < 1e-10);
}