use crate::model::geometry::geom2d::{point::Point2, line::Line2};

/// evaluate関数を使って線分との交差候補を抽出（離散近似）
pub fn sample_intersections<F>(
    evaluator: F,
    line: &Line2,
    sample_count: usize,
    epsilon: f64,
) -> Vec<Point2>
where
    F: Fn(f64) -> Point2,
{
    let mut result = vec![];
    for i in 0..sample_count {
        let theta = (i as f64) * std::f64::consts::TAU / (sample_count as f64);
        let pt = evaluator(theta);
        if line.distance_to_point(&pt) < epsilon {
            result.push(pt);
        }
    }

    result.dedup_by(|a, b| a.distance_to(b) < epsilon);
    result
}