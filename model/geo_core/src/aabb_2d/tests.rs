//! Aabb2D のテスト

use super::*;

#[test]
fn test_aabb2d_creation() {
    let min = Point2D::new(0.0, 0.0);
    let max = Point2D::new(1.0, 2.0);
    let aabb = Aabb2D::new(min, max);
    assert_eq!(aabb.min_point(), min);
    assert_eq!(aabb.max_point(), max);
}

#[test]
fn test_aabb2d_dimensions() {
    let min = Point2D::new(1.0, 2.0);
    let max = Point2D::new(4.0, 7.0);
    let aabb = Aabb2D::new(min, max);
    assert_eq!(aabb.width(), 3.0);
    assert_eq!(aabb.height(), 5.0);
    assert_eq!(aabb.area(), 15.0);
}

#[test]
fn test_aabb2d_center() {
    let min = Point2D::new(0.0, 0.0);
    let max = Point2D::new(2.0, 4.0);
    let aabb = Aabb2D::new(min, max);
    let center = aabb.center();
    assert_eq!(center.x(), 1.0);
    assert_eq!(center.y(), 2.0);
}

#[test]
fn test_contains_point() {
    let min = Point2D::new(0.0, 0.0);
    let max = Point2D::new(2.0, 2.0);
    let aabb = Aabb2D::new(min, max);
    assert!(aabb.contains_point(Point2D::new(1.0, 1.0)));
    assert!(aabb.contains_point(Point2D::new(0.0, 0.0)));
    assert!(aabb.contains_point(Point2D::new(2.0, 2.0)));
    assert!(!aabb.contains_point(Point2D::new(3.0, 1.0)));
}

#[test]
fn test_intersects() {
    let aabb1 = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0));
    let aabb2 = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
    let aabb3 = Aabb2D::new(Point2D::new(3.0, 3.0), Point2D::new(4.0, 4.0));
    assert!(aabb1.intersects(&aabb2));
    assert!(aabb2.intersects(&aabb1));
    assert!(!aabb1.intersects(&aabb3));
}

#[test]
fn test_contains_aabb() {
    let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 4.0));
    let inner = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
    let partial = Aabb2D::new(Point2D::new(2.0, 2.0), Point2D::new(5.0, 5.0));
    assert!(outer.contains_aabb(&inner));
    assert!(!outer.contains_aabb(&partial));
    assert!(!inner.contains_aabb(&outer));
}

#[test]
fn test_is_empty() {
    let valid = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
    let invalid = Aabb2D::new(Point2D::new(1.0, 0.0), Point2D::new(0.0, 1.0));
    assert!(!valid.is_empty());
    assert!(invalid.is_empty());
}

#[test]
fn test_from_points() {
    let points = vec![
        Point2D::new(1.0, 2.0),
        Point2D::new(4.0, 5.0),
        Point2D::new(0.0, 1.0),
    ];
    let aabb = Aabb2D::from_points(&points).unwrap();
    assert_eq!(aabb.min_point(), Point2D::new(0.0, 1.0));
    assert_eq!(aabb.max_point(), Point2D::new(4.0, 5.0));
}

#[test]
fn test_from_empty_points() {
    let points: Vec<Point2D<f64>> = vec![];
    assert!(Aabb2D::from_points(&points).is_none());
}

#[test]
fn test_aabb2d_capability_traits() {
    let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 6.0));
    let inner = Aabb2D::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 5.0));

    assert_eq!(outer.min_point(), Point2D::new(0.0, 0.0));
    assert_eq!(outer.max_point(), Point2D::new(4.0, 6.0));
    assert_eq!(outer.width(), 4.0);
    assert_eq!(outer.height(), 6.0);
    assert_eq!(outer.area(), 24.0);
    assert_eq!(outer.center(), Point2D::new(2.0, 3.0));
    assert!(outer.contains_point(Point2D::new(2.0, 3.0)));
    assert!(outer.intersects(&inner));
}

#[test]
fn test_contains_point_and_aabb() {
    let outer = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 4.0));
    let inner = Aabb2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));

    assert!(outer.contains_point(Point2D::new(2.0, 2.0)));
    assert!(outer.contains_aabb(&inner));
    assert!(outer.contains_point(Point2D::new(2.0, 2.0)));
    assert!(outer.contains_aabb(&inner));
}

#[test]
fn classify_point_uses_distance_to_boundary() {
    let aabb = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 2.0));
    let tolerance = 1e-3;
    let classify = |x: f64, y: f64| aabb.classify_point(Point2D::new(x, y), tolerance);

    assert_eq!(classify(2.0, 1.0), PointClassification::Inside);
    assert_eq!(classify(4.0, 1.0), PointClassification::OnBoundary);
    assert_eq!(classify(5.0, 1.0), PointClassification::Outside);

    // 辺・角までの距離が許容誤差以内なら、内側・外側のどちらからでも OnBoundary
    assert_eq!(
        classify(4.0 - 0.5 * tolerance, 1.0),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(4.0 + 0.5 * tolerance, 1.0),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(4.0 - 2.0 * tolerance, 1.0),
        PointClassification::Inside
    );
    assert_eq!(
        classify(4.0 + 2.0 * tolerance, 1.0),
        PointClassification::Outside
    );
    // 角の近くは角までの距離で判定する（各辺から 0.6 / 0.8 倍ずつ離れた点）
    assert_eq!(
        classify(4.0 + 0.6 * tolerance, 2.0 + 0.6 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(4.0 + 0.8 * tolerance, 2.0 + 0.8 * tolerance),
        PointClassification::Outside
    );

    // 境界までの距離
    assert!((aabb.distance_to_boundary(Point2D::new(1.0, 0.5)) - 0.5).abs() < 1e-12);
    assert!((aabb.distance_to_boundary(Point2D::new(7.0, 6.0)) - 5.0).abs() < 1e-12);

    // contains_point は許容誤差 0 の分類が Outside でないことと一致する
    for (x, y) in [(2.0, 1.0), (4.0, 1.0), (4.0, 2.0), (5.0, 1.0)] {
        let point = Point2D::new(x, y);
        assert_eq!(
            aabb.contains_point(point),
            aabb.classify_point(point, 0.0) != PointClassification::Outside
        );
    }

    // 既定の距離トレランスで判定する
    assert_eq!(
        aabb.classify_point(Point2D::new(0.0, 1.0), crate::default_distance_tolerance()),
        PointClassification::OnBoundary
    );
}
