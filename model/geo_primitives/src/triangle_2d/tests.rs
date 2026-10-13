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
fn test_contains_point() {
    let triangle = Triangle2D::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(2.0, 0.0),
        Point2D::new(1.0, 2.0),
    )
    .unwrap();

    // 内部の点
    assert!(triangle.contains_point(Point2D::new(1.0, 0.5)));

    // 外部の点
    assert!(!triangle.contains_point(Point2D::new(3.0, 3.0)));
}
