//! ニュートン・ラフソン法による非線形方程式求解
//!
//! 非線形方程式 f(x) = 0 の求解や逆関数計算を提供する。
//! 汎用的なニュートン法実装により、様々な数値計算問題に対応。

use crate::linalg::{
    DynamicMatrix, DynamicMatrixLinearSolver, GaussianSolver, Matrix2x2, Vector, Vector2,
};
use crate::{Scalar, DERIVATIVE_ZERO_THRESHOLD};

#[inline]
fn derivative_zero_threshold<T: Scalar>() -> T {
    T::from_f64(DERIVATIVE_ZERO_THRESHOLD)
}

#[inline]
fn solver_tolerance<T: Scalar>(tol: T) -> T {
    let threshold = derivative_zero_threshold::<T>();
    if tol < threshold {
        threshold
    } else {
        tol
    }
}

#[inline]
fn solve_linear_2x2<T: Scalar>(jacobian: Matrix2x2<T>, residual: Vector2<T>) -> Option<Vector2<T>> {
    let det = jacobian.determinant();
    if det.abs() < derivative_zero_threshold::<T>() {
        return None;
    }

    let inv_det = T::ONE / det;
    Some(Vector2::new(
        inv_det * (jacobian.get(1, 1) * residual.x() - jacobian.get(0, 1) * residual.y()),
        inv_det * (-jacobian.get(1, 0) * residual.x() + jacobian.get(0, 0) * residual.y()),
    ))
}

#[inline]
fn solve_linear_dynamic<T, S>(
    jacobian: &DynamicMatrix<T>,
    residual: &Vector<T>,
    solver: &S,
) -> Option<Vector<T>>
where
    T: Scalar,
    S: DynamicMatrixLinearSolver<T>,
{
    let solution = solver.solve_dynamic(jacobian, residual).ok()?;
    if solution.solution.len() != residual.len() {
        return None;
    }

    Some(Vector::new(solution.solution))
}

#[inline]
fn validate_multivariate_system<T: Scalar>(
    current: &Vector<T>,
    residual: &Vector<T>,
    jacobian: &DynamicMatrix<T>,
) -> bool {
    let dimension = current.len();
    residual.len() == dimension && jacobian.shape() == (dimension, dimension)
}

/// 境界付き多変数 Newton の境界条件
#[derive(Debug, Clone, PartialEq)]
pub struct MultivariateNewtonBounds<T: Scalar> {
    lower: Vector<T>,
    upper: Vector<T>,
}

impl<T: Scalar> MultivariateNewtonBounds<T> {
    pub fn new(lower: Vector<T>, upper: Vector<T>) -> Result<Self, String> {
        if lower.len() != upper.len() {
            return Err("Bounds dimension mismatch".to_string());
        }

        for index in 0..lower.len() {
            if lower[index] > upper[index] {
                return Err("Lower bound must be less than or equal to upper bound".to_string());
            }
        }

        Ok(Self { lower, upper })
    }

    pub fn dimension(&self) -> usize {
        self.lower.len()
    }

    pub fn lower(&self) -> &Vector<T> {
        &self.lower
    }

    pub fn upper(&self) -> &Vector<T> {
        &self.upper
    }

    pub fn clamp(&self, point: &Vector<T>) -> Result<Vector<T>, String> {
        if point.len() != self.dimension() {
            return Err("Point dimension mismatch".to_string());
        }

        Ok(Vector::new(
            point
                .data()
                .iter()
                .enumerate()
                .map(|(index, value)| value.clamp(self.lower[index], self.upper[index]))
                .collect(),
        ))
    }

    pub fn contains(&self, point: &Vector<T>) -> bool {
        if point.len() != self.dimension() {
            return false;
        }

        point
            .data()
            .iter()
            .enumerate()
            .all(|(index, value)| *value >= self.lower[index] && *value <= self.upper[index])
    }
}

/// 境界付き多変数 Newton の反復設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultivariateNewtonOptions<T: Scalar> {
    pub max_iter: usize,
    pub tol: T,
}

impl<T: Scalar> MultivariateNewtonOptions<T> {
    pub fn new(max_iter: usize, tol: T) -> Self {
        Self { max_iter, tol }
    }
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

/// generic 多変数連立非線形方程式をニュートン法で解く
///
/// `system` は現在の推定値 `x` を受け取り、残差ベクトルと Jacobian 行列を返す。
/// 更新ステップの線形系は既定で `GaussianSolver<T>` を用いて解く。
///
/// # Example
/// ```rust
/// use analysis::linalg::solver::newton::newton_solve_multivariate;
/// use analysis::linalg::{DynamicMatrix, Vector};
///
/// let system = |point: &Vector<f64>| {
///     let x = point[0];
///     let y = point[1];
///     let residual = Vector::new(vec![x * x + y * y - 1.0, x - y]);
///     let jacobian = DynamicMatrix::from_rows(vec![vec![2.0 * x, 2.0 * y], vec![1.0, -1.0]])
///         .unwrap();
///     (residual, jacobian)
/// };
///
/// let initial = Vector::new(vec![1.0, 0.5]);
/// let result = newton_solve_multivariate(system, initial, 100, 1e-10);
/// assert!(result.is_some());
/// ```
pub fn newton_solve_multivariate<T, F>(
    system: F,
    initial: Vector<T>,
    max_iter: usize,
    tol: T,
) -> Option<Vector<T>>
where
    T: Scalar,
    F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>),
{
    let solver = GaussianSolver::new(solver_tolerance(tol));
    newton_solve_multivariate_with_solver(system, initial, &solver, max_iter, tol)
}

/// generic 多変数連立非線形方程式を、任意の線形 solver を使ってニュートン法で解く
pub fn newton_solve_multivariate_with_solver<T, F, S>(
    system: F,
    initial: Vector<T>,
    solver: &S,
    max_iter: usize,
    tol: T,
) -> Option<Vector<T>>
where
    T: Scalar,
    F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>),
    S: DynamicMatrixLinearSolver<T>,
{
    let mut current = initial;

    for _ in 0..max_iter {
        let (residual, jacobian) = system(&current);
        if !validate_multivariate_system(&current, &residual, &jacobian) {
            return None;
        }

        if residual.norm() < tol {
            return Some(current);
        }

        let step = solve_linear_dynamic(&jacobian, &residual, solver)?;
        let step_norm = step.norm();
        let next = (current.clone() - step).ok()?;

        let (next_residual, _) = system(&next);
        if next_residual.len() != current.len() {
            return None;
        }

        if next_residual.norm() < tol && step_norm < tol {
            return Some(next);
        }

        current = next;
    }

    None
}

/// generic 境界付き多変数連立非線形方程式をニュートン法で解く
pub fn newton_solve_multivariate_bounded<T, F>(
    system: F,
    initial: Vector<T>,
    bounds: &MultivariateNewtonBounds<T>,
    options: &MultivariateNewtonOptions<T>,
) -> Option<Vector<T>>
where
    T: Scalar,
    F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>),
{
    let solver = GaussianSolver::new(solver_tolerance(options.tol));
    newton_solve_multivariate_bounded_with_solver(system, initial, bounds, &solver, options)
}

/// generic 境界付き多変数連立非線形方程式を、任意の線形 solver を使ってニュートン法で解く
pub fn newton_solve_multivariate_bounded_with_solver<T, F, S>(
    system: F,
    initial: Vector<T>,
    bounds: &MultivariateNewtonBounds<T>,
    solver: &S,
    options: &MultivariateNewtonOptions<T>,
) -> Option<Vector<T>>
where
    T: Scalar,
    F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>),
    S: DynamicMatrixLinearSolver<T>,
{
    let mut current = bounds.clamp(&initial).ok()?;

    for _ in 0..options.max_iter {
        let (residual, jacobian) = system(&current);
        if !validate_multivariate_system(&current, &residual, &jacobian) {
            return None;
        }

        if residual.norm() < options.tol {
            return Some(current);
        }

        let step = solve_linear_dynamic(&jacobian, &residual, solver)?;
        let unclamped_next = (current.clone() - step).ok()?;
        let next = bounds.clamp(&unclamped_next).ok()?;
        let actual_step = (next.clone() - current.clone()).ok()?;

        let (next_residual, _) = system(&next);
        if next_residual.len() != current.len() {
            return None;
        }

        if next_residual.norm() < options.tol && actual_step.norm() < options.tol {
            return Some(next);
        }

        current = next;
    }

    None
}

/// generic 2変数連立非線形方程式をニュートン法で解く
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
pub fn newton_solve_2d<T, F>(system: F, initial: (T, T), max_iter: usize, tol: T) -> Option<(T, T)>
where
    T: Scalar,
    F: Fn(T, T) -> (T, T, [[T; 2]; 2]),
{
    let mut x = initial.0;
    let mut y = initial.1;

    for _ in 0..max_iter {
        let (f1, f2, jacobian_raw) = system(x, y);
        let residual = Vector2::new(f1, f2);
        let jacobian = Matrix2x2::from(jacobian_raw);
        let delta = solve_linear_2x2(jacobian, residual)?;

        x -= delta.x();
        y -= delta.y();

        if residual.norm() < tol && delta.norm() < tol {
            return Some((x, y));
        }
    }

    None
}
