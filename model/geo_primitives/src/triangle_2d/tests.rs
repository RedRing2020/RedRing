//! Triangle2D のテスト

use super::*;

#[test]
fn test_triangle_creation() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(1.0, 0.0),
        Point2D::new(0.5, 1.0),
    );
    assert!(triangle.is_some());

    // 退化した三角形（一直線上の点）
    let degenerate = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(1.0, 0.0),
        Point2D::new(2.0, 0.0),
    );
    assert!(degenerate.is_none());
}

#[test]
fn test_area_calculation() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(2.0, 0.0),
        Point2D::new(1.0, 2.0),
    )
    .unwrap();

    assert_eq!(triangle.area(), 2.0);
}

#[test]
fn test_contains_point() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(2.0, 0.0),
        Point2D::new(1.0, 2.0),
    )
    .unwrap();

    // 内部の点
    assert!(triangle.contains_point(&Point2D::new(1.0, 0.5)));

    // 外部の点
    assert!(!triangle.contains_point(&Point2D::new(3.0, 3.0)));
}

#[test]
fn test_circumcenter() {
    // 単位の正三角形の外心は重心（原点）と一致
    let triangle: Triangle2D<f64> = Triangle2D::unit_triangle();
    let circumcenter = triangle.circumcenter().unwrap();

    assert!(circumcenter.x().abs() < 1e-10);
    assert!(circumcenter.y().abs() < 1e-10);
}
