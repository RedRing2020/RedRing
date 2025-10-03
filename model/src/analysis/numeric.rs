use analysis::DERIVATIVE_ZERO_THRESHOLD;

use crate::geometry_trait::normed::Normed;
use crate::geometry_trait::point_ops::PointOps;

pub fn newton_solve<F, G>(
    f: F,
    df: G,
    initial: f64,
    max_iter: usize,
    tol: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let mut x = initial;
    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);
        if dfx.abs() < DERIVATIVE_ZERO_THRESHOLD {
            return None;
        }
        let next = x - fx / dfx;
        if (next - x).abs() < tol {
            return Some(next);
        }
        x = next;
    }
    None
}

/// 単調関数 f(x) = y に対する逆関数 x をニュートン法で求める
///
/// # 引数
/// - `f`: 元の関数 f(x)
/// - `df`: f(x) の導関数（df/dx）
/// - `target`: 求めたい y 値
/// - `initial`: 初期値（x の推定値）
/// - `max_iter`: 最大反復回数
/// - `tol`: 収束判定の許容誤差
///
/// # 戻り値
/// - `Some(x)`：f(x) ≈ target を満たす x
/// - `None`：収束しない場合
pub fn newton_inverse<F, G>(
    f: F,
    df: G,
    target: f64,
    initial: f64,
    max_iter: usize,
    tol: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let g = |x: f64| f(x) - target;
    crate::analysis::numeric::newton_solve(g, df, initial, max_iter, tol)
}

pub fn find_span(n: usize, degree: usize, u: f64, knots: &[f64]) -> usize {
    if u >= knots[n + 1] {
        return n;
    }
    if u <= knots[degree] {
        return degree;
    }

    let mut low = degree;
    let mut high = n + 1;
    let mut mid = (low + high) / 2;

    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }

    mid
}

pub fn basis_functions(span: usize, u: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
    let mut N = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];

    N[0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let temp = N[r] / (right[r + 1] + left[j - r]);
            N[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        N[j] = saved;
    }

    N
}

pub fn evaluate_bspline<P: PointOps>(
    u: f64,
    degree: usize,
    control_points: &[P],
    knots: &[f64],
) -> P {
    let n = control_points.len() - 1;
    let span = find_span(n, degree, u, knots);
    let N = basis_functions(span, u, degree, knots);

    let mut result = P::origin();
    for i in 0..=degree {
        let index = span - degree + i;
        result = result.add_scaled(&control_points[index], N[i]);
    }

    result
}

pub fn evaluate_nurbs<P: PointOps>(
    u: f64,
    degree: usize,
    control_points: &[P],
    weights: &[f64],
    knots: &[f64],
) -> P {
    let n = control_points.len() - 1;
    let span = find_span(n, degree, u, knots);
    let N = basis_functions(span, u, degree, knots);

    let mut numerator = P::origin();
    let mut denominator = 0.0;

    for i in 0..=degree {
        let index = span - degree + i;
        let w = weights[index];
        let coeff = N[i] * w;
        numerator = numerator.add_scaled(&control_points[index], coeff);
        denominator += coeff;
    }

    numerator.div(denominator)
}

/// B-spline基底関数の一階導関数 Nᵢₚ′(u) を返す
pub fn basis_function_derivatives(
    span: usize,
    u: f64,
    degree: usize,
    knots: &[f64],
) -> Vec<f64> {
    let mut ders = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    let mut ndu = vec![vec![0.0; degree + 1]; degree + 1];

    ndu[0][0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let temp = ndu[r][j - 1] / (right[r + 1] + left[j - r]);
            ndu[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        ndu[j][j] = saved;
    }

    for j in 0..=degree {
        ders[j] = 0.0;
    }

    for j in 1..=degree {
        let coeff = degree as f64 / (knots[span + j] - knots[span + j - degree]);
        ders[j - 1] = coeff * (ndu[j - 1][degree - 1] - ndu[j][degree - 1]);
    }

    ders
}

/// 楕円弧の長さを数値積分で近似する関数
pub fn newton_arc_length<F, V>(evaluate: F, start: f64, end: f64, steps: usize) -> f64
where
    F: Fn(f64) -> V,
    V: Normed,
{
    let mut length = 0.0;
    let dt = (end - start) / steps as f64;

    for i in 0..steps {
        let t0 = start + i as f64 * dt;
        let t1 = t0 + dt;

        let v0 = evaluate(t0);
        let v1 = evaluate(t1);

        length += 0.5 * (v0.norm() + v1.norm()) * dt;
    }

    length
}
