use crate::statistics::*;
use geo_core::{Point2D, ToleranceContext};

#[test]
fn test_basic_stats() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let stats = BasicStats::from_f64_slice(&values);
    
    assert!((stats.mean - 3.0).abs() < 1e-10);
    assert!((stats.variance - 2.0).abs() < 1e-10);
    assert_eq!(stats.count, 5);
}

#[test]
fn test_centroid() {
    let tolerance = ToleranceContext::standard();
    let analyzer = PointCloudStats::new(tolerance);
    
    let points = vec![
        Point2D::from_f64(0.0, 0.0),
        Point2D::from_f64(2.0, 0.0),
        Point2D::from_f64(1.0, 2.0),
    ];

    let centroid = analyzer.centroid(&points).unwrap();
    assert!((centroid.x().value() - 1.0).abs() < 1e-10);
    assert!((centroid.y().value() - 2.0/3.0).abs() < 1e-10);
}