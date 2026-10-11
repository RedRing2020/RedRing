//! Rect3D のテスト

use super::*;

const POINT_CONTAINMENT_TOLERANCE: f64 = 1e-9;

#[test]
fn contains_and_resize() {
    let mut rect = Rect3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::unit_x(),
        Vector3D::unit_y(),
        10.0,
        5.0,
    )
    .unwrap();

    assert!(rect.contains_point(&Point3D::new(5.0, 2.5, 0.0), POINT_CONTAINMENT_TOLERANCE,));
    assert!(!rect.contains_point(&Point3D::new(11.0, 2.5, 0.0), POINT_CONTAINMENT_TOLERANCE,));
    assert!(!rect.contains_point(&Point3D::new(5.0, 2.5, 0.1), 1e-3));

    assert!(rect.resize(20.0, 10.0));
    assert_eq!(rect.width_value(), 20.0);
    assert_eq!(rect.height_value(), 10.0);
    assert!(!rect.resize(-1.0, 10.0));
}
