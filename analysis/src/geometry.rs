//! 幾何特化数値計算モジュール
//!
//! NURBS曲線、弧長計算など、幾何形状に特化した数値計算関数を提供する。
//! 元々model/src/analysis/numeric.rsにあった関数群を独立化。
use crate::DERIVATIVE_ZERO_THRESHOLD;

/// ニュートン法による方程式求解
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
    newton_solve(g, df, initial, max_iter, tol)
}

/// NURBS/B-splineノットベクトルからスパンインデックスを検索
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

/// B-spline基底関数Nᵢₚ(u)の値を計算
pub fn basis_functions(span: usize, u: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
    let mut n = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];

    n[0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let temp = n[r] / (right[r + 1] + left[j - r]);
            n[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        n[j] = saved;
    }

    n
}

/// B-spline基底関数の一階導関数 Nᵢₚ′(u) を計算
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

    #[allow(clippy::needless_range_loop)]
    for j in 0..=degree {
        ders[j] = 0.0;
    }

    for j in 1..=degree {
        let coeff = degree as f64 / (knots[span + j] - knots[span + j - degree]);
        ders[j - 1] = coeff * (ndu[j - 1][degree - 1] - ndu[j][degree - 1]);
    }

    ders
}

/// 曲線の弧長を数値積分で近似計算（model::geometry::Vector向け）
pub fn newton_arc_length<F, V>(evaluate: F, start: f64, end: f64, steps: usize) -> f64
where
    F: Fn(f64) -> V,
    V: NormedVector,
{
    let mut length = 0.0;
    let dt = (end - start) / steps as f64;

    for i in 0..steps {
        let t0 = start + i as f64 * dt;
        let t1 = t0 + dt;

        let v0 = evaluate(t0);
        let v1 = evaluate(t1);

        // ベクトルの大きさを計算
        length += 0.5 * (v0.norm() + v1.norm()) * dt;
    }

    length
}

/// 弧長計算で使用するベクトルの共通インターフェース
pub trait NormedVector {
    fn norm(&self) -> f64;
}
