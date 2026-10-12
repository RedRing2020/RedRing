//! Aabb3D のテスト

use super::*;
use geo_contracts::{Aabb3DDerived, Aabb3DProperties, Aabb3DRelation, Contains};

#[test]
fn test_aabb3d_creation() {
    let min = Point3D::new(0.0, 0.0, 0.0);
    let max = Point3D::new(1.0, 2.0, 3.0);
    let aabb = Aabb3D::new(min, max);
    assert_eq!(aabb.min(), min);
    assert_eq!(aabb.max(), max);
}

#[test]
fn test_aabb3d_dimensions() {
    let min = Point3D::new(1.0, 2.0, 3.0);
    let max = Point3D::new(4.0, 7.0, 9.0);
    let aabb = Aabb3D::new(min, max);
    assert_eq!(aabb.width(), 3.0);
    assert_eq!(aabb.height(), 5.0);
    assert_eq!(aabb.depth(), 6.0);
    assert_eq!(aabb.volume(), 90.0);
}

#[test]
fn test_aabb3d_center() {
    let min = Point3D::new(0.0, 0.0, 0.0);
    let max = Point3D::new(2.0, 4.0, 6.0);
    let aabb = Aabb3D::new(min, max);
    let center = aabb.center();
    assert_eq!(center.x(), 1.0);
    assert_eq!(center.y(), 2.0);
    assert_eq!(center.z(), 3.0);
}

#[test]
fn test_contains() {
    let min = Point3D::new(0.0, 0.0, 0.0);
    let max = Point3D::new(2.0, 2.0, 2.0);
    let aabb = Aabb3D::new(min, max);
    assert!(aabb.contains_point(&Point3D::new(1.0, 1.0, 1.0)));
    assert!(aabb.contains_point(&Point3D::new(0.0, 0.0, 0.0)));
    assert!(aabb.contains_point(&Point3D::new(2.0, 2.0, 2.0)));
    assert!(!aabb.contains_point(&Point3D::new(3.0, 1.0, 1.0)));
}

#[test]
fn test_from_points() {
    let points = vec![
        Point3D::new(1.0, 2.0, 3.0),
        Point3D::new(4.0, 5.0, 6.0),
        Point3D::new(0.0, 1.0, 2.0),
    ];
    let aabb = Aabb3D::from_points(&points).unwrap();
    assert_eq!(aabb.min(), Point3D::new(0.0, 1.0, 2.0));
    assert_eq!(aabb.max(), Point3D::new(4.0, 5.0, 6.0));
}

#[test]
fn test_is_empty() {
    let valid = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0));
    let invalid = Aabb3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 1.0));
    assert!(!valid.is_empty());
    assert!(invalid.is_empty());
}

#[test]
fn test_aabb3d_capability_traits() {
    let outer = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 6.0, 8.0));
    let inner = Aabb3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(3.0, 5.0, 7.0));

    assert_eq!(Aabb3DProperties::min(&outer), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(Aabb3DProperties::max(&outer), Point3D::new(4.0, 6.0, 8.0));
    assert_eq!(Aabb3DDerived::width(&outer), 4.0);
    assert_eq!(Aabb3DDerived::height(&outer), 6.0);
    assert_eq!(Aabb3DDerived::depth(&outer), 8.0);
    assert_eq!(Aabb3DDerived::volume(&outer), 192.0);
    assert_eq!(Aabb3DDerived::center(&outer), Point3D::new(2.0, 3.0, 4.0));
    assert!(Aabb3DDerived::is_valid(&outer));
    assert!(Aabb3DRelation::contains_point(
        &outer,
        &Point3D::new(2.0, 3.0, 4.0)
    ));
    assert!(Aabb3DRelation::intersects(&outer, &inner));
}

#[test]
fn test_contains_trait_for_point_and_aabb() {
    let outer = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 4.0, 4.0));
    let inner = Aabb3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 3.0, 3.0));

    assert!(Contains::contains(&outer, &Point3D::new(2.0, 2.0, 2.0)));
    assert!(Contains::contains(&outer, &inner));
    assert!(outer.contains(&Point3D::new(2.0, 2.0, 2.0)));
    assert!(outer.contains(&inner));
}

#[test]
fn classify_point_uses_distance_to_boundary() {
    use geo_contracts::PointClassification;

    let aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 2.0, 2.0));
    let tolerance = 1e-3;
    let classify = |x: f64, y: f64, z: f64| aabb.classify_point(&Point3D::new(x, y, z), tolerance);

    assert_eq!(classify(2.0, 1.0, 1.0), PointClassification::Inside);
    assert_eq!(classify(2.0, 1.0, 2.0), PointClassification::OnBoundary);
    assert_eq!(classify(2.0, 1.0, 3.0), PointClassification::Outside);
    assert_eq!(
        classify(2.0, 1.0, 2.0 - 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(2.0, 1.0, 2.0 + 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(2.0, 1.0, 2.0 - 2.0 * tolerance),
        PointClassification::Inside
    );

    // 境界までの距離
    assert!((aabb.distance_to_boundary(&Point3D::new(2.0, 1.0, 0.5)) - 0.5).abs() < 1e-12);
    assert!((aabb.distance_to_boundary(&Point3D::new(6.0, 2.0, 0.0)) - 2.0).abs() < 1e-12);

    // contains_point は許容誤差 0 の分類が Outside でないことと一致する
    for (x, y, z) in [(2.0, 1.0, 1.0), (4.0, 2.0, 2.0), (4.5, 1.0, 1.0)] {
        let point = Point3D::new(x, y, z);
        assert_eq!(
            aabb.contains_point(&point),
            aabb.classify_point(&point, 0.0) != PointClassification::Outside
        );
    }

    assert_eq!(
        Aabb3DRelation::classify_point(&aabb, &Point3D::new(1.0, 1.0, 1.0)),
        PointClassification::Inside
    );
}
