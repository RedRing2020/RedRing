//! Rect3D のテスト

use super::*;

#[test]
fn contains_point() {
    let rect = Rect3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::positive_x(),
        Direction3D::new(0.0, 1.0, 0.0).unwrap(),
        10.0,
        5.0,
    )
    .unwrap();

    assert!(rect.contains_point(Point3D::new(5.0, 2.5, 0.0)));
    assert!(!rect.contains_point(Point3D::new(11.0, 2.5, 0.0)));
    assert!(!rect.contains_point(Point3D::new(5.0, 2.5, 0.1)));
}
