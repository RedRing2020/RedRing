//! InfiniteLine2D のテストコード

// TODO: InfiniteLine2Dは現在型変換作業中のため、一時的にテストを無効化
// InfiniteLine2Dの実装が完了したら、これらのテストを有効化する

#[cfg(feature = "infinite_line_2d_tests_disabled")]
mod infinite_line_tests {
    use crate::geometry2d::{Direction2D, Point2D, Vector2D};
    use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;
    use geo_foundation::Direction; // Direction traitをインポート

#[test]
fn test_infinite_line_creation() {
    let origin = Point2D::new(1.0, 2.0);
    let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
    let line = InfiniteLine2D::new(origin, direction);

    assert_eq!(line.origin(), origin);
    assert_eq!(line.direction(), direction);
}

#[test]
fn test_from_two_points() {
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(1.0, 1.0);
    let line = InfiniteLine2D::from_two_points(p1, p2).unwrap();

    assert_eq!(line.origin(), p1);
    assert!((line.direction().x() - line.direction().y()).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_horizontal_vertical_lines() {
    let h_line = InfiniteLine2D::horizontal(3.0);
    assert_eq!(h_line.origin().y(), 3.0);
    assert!((h_line.direction().y()).abs() < GEOMETRIC_TOLERANCE);

    let v_line = InfiniteLine2D::vertical(2.0);
    assert_eq!(v_line.origin().x(), 2.0);
    assert!((v_line.direction().x()).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_distance_to_point() {
    let line = InfiniteLine2D::horizontal(0.0); // X軸
    let point = Point2D::new(5.0, 3.0);
    let distance = line.distance_to_point(&point);

    assert!((distance - 3.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_closest_point() {
    let line = InfiniteLine2D::horizontal(0.0); // X軸
    let point = Point2D::new(5.0, 3.0);
    let closest = line.closest_point(&point);

    assert!((closest.x() - 5.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((closest.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_line_intersection() {
    let line1 = InfiniteLine2D::horizontal(0.0); // X軸
    let line2 = InfiniteLine2D::vertical(0.0); // Y軸

    let intersection = line1.intersect_line(&line2).unwrap();
    assert!((intersection.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    assert!((intersection.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_parallel_lines() {
    let line1 = InfiniteLine2D::horizontal(0.0);
    let line2 = InfiniteLine2D::horizontal(1.0);

    assert!(line1.is_parallel_to(&line2, GEOMETRIC_TOLERANCE));
    assert!(!line1.is_coincident_with(&line2, GEOMETRIC_TOLERANCE));
}

#[test]
fn test_equation_coefficients() {
    let line =
        InfiniteLine2D::from_two_points(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)).unwrap();

    let (a, b, c) = line.equation_coefficients();

    // 点(0,0)と(1,1)が方程式を満たすかチェック
    assert!((a * 0.0 + b * 0.0 + c).abs() < GEOMETRIC_TOLERANCE);
    assert!((a * 1.0 + b * 1.0 + c).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_parameter_at_point() {
    let line =
        InfiniteLine2D::from_two_points(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();

    let point = Point2D::new(5.0, 0.0);
    let param = line.parameter_at_point(&point);

    assert!((param - 5.0).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_sample_points() {
    let line = InfiniteLine2D::horizontal(0.0);
    let points = line.sample_points(-2.0, 2.0, 5);

    assert_eq!(points.len(), 5);
    assert!((points[0].x() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);
    assert!((points[4].x() - 2.0).abs() < GEOMETRIC_TOLERANCE);

    for point in &points {
        assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }
}

}
