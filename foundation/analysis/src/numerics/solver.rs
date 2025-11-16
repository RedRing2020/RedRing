//! 数値計算手法モジュール
//!
//! 汎用数値計算手法とNURBS/B-spline特化機能を提供する。
//! 元々model/src/analysis/numeric.rsにあった関数群を独立化し、
//! より適切な名前と構造に整理。

use crate::DERIVATIVE_ZERO_THRESHOLD;

// =============================================================================
// 汎用数値計算手法 (General Numerical Methods)
// =============================================================================

/// ニュートン法による方程式求解
///
/// 一般的な非線形方程式 f(x) = 0 をニュートン・ラフソン法で解く。
/// 初期値と関数、その導関数を指定して反復計算を行う。
pub fn newton_solve<F, G>(f: F, df: G, initial: f64, max_iter: usize, tol: f64) -> Option<f64>
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
/// 既知の関数値 y に対して、f(x) = y を満たす x を求める。
/// 単調関数（狭義単調増加または狭義単調減少）である必要がある。
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

/// 曲線の弧長を数値積分で近似計算（汎用ベクトル向け）
///
/// パラメトリック曲線 r(t) の弧長を台形公式により数値積分で計算。
/// 任意のベクトル型に対応するため、NormedVectorトレイトを使用。
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

        // ベクトルの大きさを計算（台形公式）
        length += 0.5 * (v0.norm() + v1.norm()) * dt;
    }

    length
}

/// 弧長計算で使用するベクトルの共通インターフェース
///
/// 異なるベクトル実装に対して統一的なノルム計算を提供。
/// 2D、3D、N次元ベクトルなど、任意の次元に対応可能。
pub trait NormedVector {
    fn norm(&self) -> f64;
}

// =============================================================================
// 注意: B-spline/NURBS特化機能は geo_nurbs クレートに移動済み
// =============================================================================
//
// 以前ここにあった以下の機能は geo_nurbs/basis.rs に統合されています：
// - find_span() - ノットスパン検索
// - basis_functions() - B-spline基底関数計算
// - basis_function_derivatives() - 基底関数導関数計算
//
// これらの機能は形状特化機能であり、analysisクレートの責務範囲外です。
// 新しい使用方法：
//   use geo_nurbs::basis::{basis_function, basis_functions, rational_basis_functions};
