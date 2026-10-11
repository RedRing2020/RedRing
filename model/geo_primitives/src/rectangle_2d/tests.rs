//! Rect2D のテスト

use super::*;

#[test]
fn contains_and_resize() {
    let mut rect = Rect2D::new(Point2D::new(10.0, 20.0), 30.0, 40.0).unwrap();
    assert!(rect.contains_point(&Point2D::new(10.0, 20.0)));
    assert!(rect.contains_point(&Point2D::new(40.0, 60.0)));
    assert!(!rect.contains_point(&Point2D::new(41.0, 60.0)));

    assert!(rect.resize(50.0, 10.0));
    assert_eq!(rect.width_value(), 50.0);
    assert_eq!(rect.height_value(), 10.0);
    assert!(!rect.resize(-1.0, 10.0));
}
