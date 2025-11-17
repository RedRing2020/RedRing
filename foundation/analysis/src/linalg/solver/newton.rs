//! ニュートン・ラフソン法による非線形方程式求解
//!
//! 非線形方程式 f(x) = 0 の求解や逆関数計算を提供する。
//! 汎用的なニュートン法実装により、様々な数値計算問題に対応。

use crate::DERIVATIVE_ZERO_THRESHOLD;

/// ニュートン法による方程式求解
///
/// 一般的な非線形方程式 f(x) = 0 をニュートン・ラフソン法で解く。
/// 初期値と関数、その導関数を指定して反復計算を行う。
///
/// # Arguments
/// * `f` - 解きたい方程式 f(x) = 0 の関数
/// * `df` - f(x) の導関数
/// * `initial` - 反復の初期値
/// * `max_iter` - 最大反復回数
/// * `tol` - 収束判定の許容誤差
///
/// # Returns
/// * `Some(x)` - 収束した場合の解
/// * `None` - 発散または導関数が0になった場合
///
/// # Example
/// ```rust
/// use analysis::linalg::solver::newton::newton_solve;
/// 
/// // x^2 - 2 = 0 の解を求める（√2を計算）
/// let f = |x: f64| x * x - 2.0;
/// let df = |x: f64| 2.0 * x;
/// let result = newton_solve(f, df, 1.0, 100, 1e-10);
/// assert!((result.unwrap() - 1.41421356).abs() < 1e-6);
/// ```
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
///
/// # Arguments
/// * `f` - 逆関数を求めたい単調関数
/// * `df` - f(x) の導関数
/// * `target` - 目標値 y
/// * `initial` - 反復の初期値
/// * `max_iter` - 最大反復回数
/// * `tol` - 収束判定の許容誤差
///
/// # Returns
/// * `Some(x)` - f(x) = target を満たす x
/// * `None` - 収束しなかった場合
///
/// # Example
/// ```rust
/// use analysis::linalg::solver::newton::newton_inverse;
/// 
/// // x^3 の逆関数（立方根）を計算
/// let f = |x: f64| x * x * x;
/// let df = |x: f64| 3.0 * x * x;
/// let result = newton_inverse(f, df, 8.0, 2.0, 100, 1e-10);
/// assert!((result.unwrap() - 2.0).abs() < 1e-6);
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newton_solve_square_root() {
        // x^2 - 2 = 0 の解を求める（√2を計算）
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        let result = newton_solve(f, df, 1.0, 100, 1e-10);
        
        assert!(result.is_some());
        let sqrt_2 = result.unwrap();
        assert!((sqrt_2 - 1.41421356).abs() < 1e-6);
    }

    #[test]
    fn test_newton_inverse_cube_root() {
        // x^3 の逆関数（立方根）を計算
        let f = |x: f64| x * x * x;
        let df = |x: f64| 3.0 * x * x;
        let result = newton_inverse(f, df, 8.0, 2.0, 100, 1e-10);
        
        assert!(result.is_some());
        let cube_root = result.unwrap();
        assert!((cube_root - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_newton_solve_zero_derivative() {
        // 導関数が0になるケース
        let f = |x: f64| x * x;
        let df = |_: f64| 0.0; // 常に0の導関数
        let result = newton_solve(f, df, 1.0, 100, 1e-10);
        
        assert!(result.is_none());
    }
}