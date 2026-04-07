//! ニュートン・ラフソン法による非線形方程式求解
//!
//! 非線形方程式 f(x) = 0 の求解や逆関数計算を提供する。
//! 汎用的なニュートン法実装により、様々な数値計算問題に対応。

use crate::{Scalar, DERIVATIVE_ZERO_THRESHOLD};

#[inline]
fn derivative_zero_threshold<T: Scalar>() -> T {
    T::from_f64(DERIVATIVE_ZERO_THRESHOLD)
}

/// generic Newton 法による方程式求解
pub fn newton_solve_generic<T, F, G>(f: F, df: G, initial: T, max_iter: usize, tol: T) -> Option<T>
where
    T: Scalar,
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    let mut x = initial;
    let derivative_threshold = derivative_zero_threshold::<T>();

    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);
        if dfx.abs() < derivative_threshold {
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

/// generic 境界付き Newton 法による方程式求解
pub fn newton_solve_bounded_generic<T, F, G>(
    f: F,
    df: G,
    initial: T,
    min: T,
    max: T,
    max_iter: usize,
    tol: T,
) -> Option<T>
where
    T: Scalar,
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    let mut x = initial.clamp(min, max);
    let derivative_threshold = derivative_zero_threshold::<T>();

    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);
        if dfx.abs() < derivative_threshold {
            return None;
        }

        let next = (x - fx / dfx).clamp(min, max);
        if (next - x).abs() < tol {
            return Some(next);
        }
        x = next;
    }
    None
}

/// generic 数値微分（前進差分）を用いた境界付き Newton 法
pub fn newton_solve_with_numeric_derivative_bounded_generic<T, F>(
    f: F,
    initial: T,
    min: T,
    max: T,
    max_iter: usize,
    tol: T,
    diff_step: T,
) -> Option<T>
where
    T: Scalar,
    F: Fn(T) -> T,
{
    let derivative_threshold = derivative_zero_threshold::<T>();
    let df = |x: T| {
        let h = if diff_step.abs() < derivative_threshold {
            derivative_threshold
        } else {
            diff_step
        };
        let x_plus = (x + h).min(max);
        let fx = f(x);
        let fx_plus = f(x_plus);
        let effective_h = (x_plus - x).abs();

        if effective_h < derivative_threshold {
            T::ZERO
        } else {
            (fx_plus - fx) / effective_h
        }
    };

    newton_solve_bounded_generic(&f, df, initial, min, max, max_iter, tol)
}

/// generic 単調関数 f(x) = y に対する逆関数 x を Newton 法で求める
pub fn newton_inverse_generic<T, F, G>(
    f: F,
    df: G,
    target: T,
    initial: T,
    max_iter: usize,
    tol: T,
) -> Option<T>
where
    T: Scalar,
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    let g = |x: T| f(x) - target;
    newton_solve_generic(g, df, initial, max_iter, tol)
}

/// ニュートン法による方程式求解
///
/// `f64` 固定の互換 API。
/// generic 実装を使う新規コードでは `newton_solve_generic` を優先する。
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
/// use analysis::linalg::solver::newton::newton_solve_generic;
///
/// // x^2 - 2 = 0 の解を求める（√2を計算）
/// let f = |x: f64| x * x - 2.0;
/// let df = |x: f64| 2.0 * x;
/// let result = newton_solve_generic(f, df, 1.0, 100, 1e-10);
/// assert!((result.unwrap() - std::f64::consts::SQRT_2).abs() < 1e-6);
/// ```
pub fn newton_solve<F, G>(f: F, df: G, initial: f64, max_iter: usize, tol: f64) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    newton_solve_generic(f, df, initial, max_iter, tol)
}

/// 境界付きニュートン法による方程式求解
///
/// `f64` 固定の互換 API。
/// generic 実装を使う新規コードでは `newton_solve_bounded_generic` を優先する。
///
/// `newton_solve` と同様に f(x)=0 を解くが、各反復更新後に値を `[min, max]` にクランプする。
/// パラメータ領域が有限な問題（例: NURBS パラメータ最適化）で使用する。
pub fn newton_solve_bounded<F, G>(
    f: F,
    df: G,
    initial: f64,
    min: f64,
    max: f64,
    max_iter: usize,
    tol: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    newton_solve_bounded_generic(f, df, initial, min, max, max_iter, tol)
}

/// 数値微分（前進差分）を用いた境界付きニュートン法
///
/// `f64` 固定の互換 API。
/// generic 実装を使う新規コードでは
/// `newton_solve_with_numeric_derivative_bounded_generic` を優先する。
///
/// 導関数を解析的に与えづらい場合に、`f(x+h)-f(x)` の差分で導関数を近似して解く。
/// 反復制御と境界拘束は `newton_solve_bounded` に委譲する。
pub fn newton_solve_with_numeric_derivative_bounded<F>(
    f: F,
    initial: f64,
    min: f64,
    max: f64,
    max_iter: usize,
    tol: f64,
    diff_step: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    newton_solve_with_numeric_derivative_bounded_generic(
        f, initial, min, max, max_iter, tol, diff_step,
    )
}

/// 単調関数 f(x) = y に対する逆関数 x をニュートン法で求める
///
/// `f64` 固定の互換 API。
/// generic 実装を使う新規コードでは `newton_inverse_generic` を優先する。
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
/// use analysis::linalg::solver::newton::newton_inverse_generic;
///
/// // x^3 の逆関数（立方根）を計算
/// let f = |x: f64| x * x * x;
/// let df = |x: f64| 3.0 * x * x;
/// let result = newton_inverse_generic(f, df, 8.0, 2.0, 100, 1e-10);
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
    newton_inverse_generic(f, df, target, initial, max_iter, tol)
}

/// 2変数連立非線形方程式をニュートン法で解く
///
/// f1(x, y) = 0 と f2(x, y) = 0 を同時に満たす (x, y) を求める。
/// ヤコビ行列を用いた多変数ニュートン法を実装。
///
/// # Arguments
/// * `system` - (f1, f2, ヤコビ行列) を返す関数
///   - f1, f2: 方程式の値
///   - jacobian: [[∂f1/∂x, ∂f1/∂y], [∂f2/∂x, ∂f2/∂y]]
/// * `initial` - 初期値 (x0, y0)
/// * `max_iter` - 最大反復回数
/// * `tol` - 収束判定の許容誤差
///
/// # Returns
/// * `Some((x, y))` - 収束した場合の解
/// * `None` - 発散またはヤコビ行列が特異な場合
///
/// # Example
/// ```rust
/// use analysis::linalg::solver::newton::newton_solve_2d;
///
/// // 連立方程式: x^2 + y^2 = 1, x - y = 0 (単位円と y=x の交点)
/// let system = |x: f64, y: f64| {
///     let f1 = x * x + y * y - 1.0;
///     let f2 = x - y;
///     let jacobian = [
///         [2.0 * x, 2.0 * y],
///         [1.0, -1.0]
///     ];
///     (f1, f2, jacobian)
/// };
/// let result = newton_solve_2d(system, (1.0, 0.5), 100, 1e-10);
/// assert!(result.is_some());
/// let (x, y) = result.unwrap();
/// let expected = 1.0 / 2_f64.sqrt();
/// assert!((x - expected).abs() < 1e-6);
/// assert!((y - expected).abs() < 1e-6);
/// ```
pub fn newton_solve_2d<F>(
    system: F,
    initial: (f64, f64),
    max_iter: usize,
    tol: f64,
) -> Option<(f64, f64)>
where
    F: Fn(f64, f64) -> (f64, f64, [[f64; 2]; 2]),
{
    let mut x = initial.0;
    let mut y = initial.1;

    for _ in 0..max_iter {
        let (f1, f2, jacobian) = system(x, y);

        // ヤコビ行列の行列式を計算
        let det = jacobian[0][0] * jacobian[1][1] - jacobian[0][1] * jacobian[1][0];

        if det.abs() < DERIVATIVE_ZERO_THRESHOLD {
            return None; // 特異行列
        }

        // クラメルの公式で逆行列を計算して解を更新
        let inv_det = 1.0 / det;
        let dx = inv_det * (jacobian[1][1] * f1 - jacobian[0][1] * f2);
        let dy = inv_det * (-jacobian[1][0] * f1 + jacobian[0][0] * f2);

        x -= dx;
        y -= dy;

        // 収束判定
        let residual = (f1 * f1 + f2 * f2).sqrt();
        let step_size = (dx * dx + dy * dy).sqrt();

        if residual < tol && step_size < tol {
            return Some((x, y));
        }
    }

    None
}
