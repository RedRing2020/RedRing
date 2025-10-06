use geo_algorithms::sampling::*;
use geo_core::{Point2D, ToleranceContext};

#[test]
fn test_adaptive_sampler() {
    let tolerance = ToleranceContext::standard();
    let sampler = AdaptiveSampler::new(tolerance);

    let circle_evaluator = |t: f64| Point2D::from_f64(t.cos(), t.sin());
    let curvature_fn = |_t: f64| 1.0; // 円の曲率は1

    let result = sampler.sample_curve_adaptive(
        circle_evaluator,
        curvature_fn,
        0.0,
        2.0 * std::f64::consts::PI,
    );

    assert!(!result.points.is_empty());
    assert!(result.quality_metrics.uniformity_score > 0.0);
}

#[test]
fn test_poisson_disk_sampler() {
    let sampler = PoissonDiskSampler::new(0.1);
    let bounds = (Point2D::from_f64(0.0, 0.0), Point2D::from_f64(1.0, 1.0));
    
    let points = sampler.sample_uniform_2d(bounds, None);
    
    assert!(!points.is_empty());
    
    // 距離制約をチェック
    for i in 0..points.len() {
        for j in i+1..points.len() {
            let distance = points[i].distance_to(&points[j]).value();
            assert!(distance >= 0.1, "Points too close: {}", distance);
        }
    }
}