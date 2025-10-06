// use geo_core::{Point2D as Point, LineSegment2D};  // 一時的にコメントアウト
// use geo_primitives::Point2D as Point;  // 一時的にコメントアウト

// /// evaluate関数を使って線分との交差候補を抽出（離散近似）
// pub fn sample_intersections<F>(
//     evaluator: F,
//     line: &LineSegment2D,
//     sample_count: usize,
//     epsilon: f64,
// ) -> Vec<Point>
// where
//     F: Fn(f64) -> Point,
// {
//     let mut result = vec![];
//     for i in 0..sample_count {
//         let theta = (i as f64) * std::f64::consts::TAU / (sample_count as f64);
//         let pt = evaluator(theta);
//         if line.distance_to_point(&pt).value() < epsilon {
//             result.push(pt);
//         }
//     }
//
//     result.dedup_by(|a, b| a.distance_to(b).value() < epsilon);
//     result
// }
