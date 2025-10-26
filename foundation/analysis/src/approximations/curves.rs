//! 曲線長計算の近似

use crate::abstract_types::Scalar;

/// ベジェ曲線の長さ近似（分割近似法）
pub fn bezier_length_approximation<T: Scalar>(control_points: &[[T; 2]], num_segments: usize) -> T {
    if control_points.len() < 2 || num_segments == 0 {
        return T::ZERO;
    }

    let mut total_length = T::ZERO;
    let step = T::ONE / T::from_usize(num_segments);

    for i in 0..num_segments {
        let t1 = T::from_usize(i) * step;
        let t2 = T::from_usize(i + 1) * step;

        let p1 = evaluate_bezier(control_points, t1);
        let p2 = evaluate_bezier(control_points, t2);

        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        total_length += (dx * dx + dy * dy).sqrt();
    }

    total_length
}

/// ベジェ曲線上の点を計算
fn evaluate_bezier<T: Scalar>(control_points: &[[T; 2]], t: T) -> [T; 2] {
    let n = control_points.len();
    if n == 1 {
        return control_points[0];
    }

    // 線形補間で近似（簡略版）
    if n == 2 {
        let x = control_points[0][0] * (T::ONE - t) + control_points[1][0] * t;
        let y = control_points[0][1] * (T::ONE - t) + control_points[1][1] * t;
        return [x, y];
    }

    // 3次ベジェ曲線の場合
    if n == 4 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = T::ONE - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x = control_points[0][0] * mt3
            + T::from_f64(3.0) * control_points[1][0] * mt2 * t
            + T::from_f64(3.0) * control_points[2][0] * mt * t2
            + control_points[3][0] * t3;

        let y = control_points[0][1] * mt3
            + T::from_f64(3.0) * control_points[1][1] * mt2 * t
            + T::from_f64(3.0) * control_points[2][1] * mt * t2
            + control_points[3][1] * t3;

        return [x, y];
    }

    // その他の場合は線形補間で近似
    let x = control_points[0][0] * (T::ONE - t) + control_points[n - 1][0] * t;
    let y = control_points[0][1] * (T::ONE - t) + control_points[n - 1][1] * t;
    [x, y]
}

/// スプライン曲線の長さ近似
pub fn spline_length_approximation<T: Scalar>(
    points: &[[T; 2]],
    _num_segments_per_span: usize,
) -> T {
    if points.len() < 2 {
        return T::ZERO;
    }

    // 簡単な近似：各点間を直線で結んだ長さ
    let mut total_length = T::ZERO;
    for i in 0..points.len() - 1 {
        let dx = points[i + 1][0] - points[i][0];
        let dy = points[i + 1][1] - points[i][1];
        total_length += (dx * dx + dy * dy).sqrt();
    }

    // スプライン補正係数（経験的な値）
    total_length * T::from_f64(1.1)
}

/// パラメトリック曲線の長さ近似（数値積分）
pub fn parametric_curve_length<T, F>(curve_fn: F, t_start: T, t_end: T, num_segments: usize) -> T
where
    T: Scalar,
    F: Fn(T) -> [T; 2],
{
    if num_segments == 0 {
        return T::ZERO;
    }

    let mut total_length = T::ZERO;
    let step = (t_end - t_start) / T::from_usize(num_segments);

    for i in 0..num_segments {
        let t1 = t_start + T::from_usize(i) * step;
        let t2 = t_start + T::from_usize(i + 1) * step;

        let p1 = curve_fn(t1);
        let p2 = curve_fn(t2);

        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        total_length += (dx * dx + dy * dy).sqrt();
    }

    total_length
}
