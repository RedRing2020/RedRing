//! ニュートン・ラフソン法による非線形方程式求解
//!
//! 非線形方程式 f(x) = 0 の求解や逆関数計算を提供する。
//! 汎用的なニュートン法実装により、様々な数値計算問題に対応。

use std::any::TypeId;

use crate::consts::numerical::{
    DERIVATIVE_ZERO_THRESHOLD_F32, DERIVATIVE_ZERO_THRESHOLD_F64, LINEAR_SOLVER_TOLERANCE_F32,
    LINEAR_SOLVER_TOLERANCE_F64,
};
use crate::linalg::{DynamicMatrix, GaussianSolver, LinearSolver, Vector};
use crate::Scalar;

#[inline]
fn derivative_zero_threshold<T: Scalar>() -> T {
    if TypeId::of::<T>() == TypeId::of::<f32>() {
        T::from_f32(DERIVATIVE_ZERO_THRESHOLD_F32)
    } else {
        T::from_f64(DERIVATIVE_ZERO_THRESHOLD_F64)
    }
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
fn default_linear_solver_tolerance<T: Scalar>() -> T {
    let tolerance = if TypeId::of::<T>() == TypeId::of::<f32>() {
        T::from_f32(LINEAR_SOLVER_TOLERANCE_F32)
    } else {
        T::from_f64(LINEAR_SOLVER_TOLERANCE_F64)
    };

    solver_tolerance(tolerance)
}

#[inline]
fn solve_linear_dynamic<T, S>(
    jacobian: &DynamicMatrix<T>,
    residual: &Vector<T>,
    solver: &S,
) -> Option<Vector<T>>
where
    T: Scalar,
    S: LinearSolver<T>,
{
    let solution = solver.solve(jacobian, residual.data()).ok()?;
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

    pub fn contains_with_tolerance(&self, point: &Vector<T>, tol: T) -> bool {
        if point.len() != self.dimension() {
            return false;
        }

        let effective_tol = tol.max(T::ZERO);

        point.data().iter().enumerate().all(|(index, value)| {
            *value >= self.lower[index] - effective_tol
                && *value <= self.upper[index] + effective_tol
        })
    }
}

/// 境界付き多変数 Newton の反復設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultivariateNewtonOptions<T: Scalar> {
    pub max_iter: usize,
    pub residual_tol: T,
    pub step_tol: T,
}

impl<T: Scalar> MultivariateNewtonOptions<T> {
    pub fn new(max_iter: usize, residual_tol: T, step_tol: T) -> Self {
        Self {
            max_iter,
            residual_tol,
            step_tol,
        }
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
    let solver = GaussianSolver::new(default_linear_solver_tolerance());
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
    S: LinearSolver<T>,
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
    let solver = GaussianSolver::new(default_linear_solver_tolerance());
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
    S: LinearSolver<T>,
{
    let mut current = bounds.clamp(&initial).ok()?;

    for _ in 0..options.max_iter {
        let (residual, jacobian) = system(&current);
        if !validate_multivariate_system(&current, &residual, &jacobian) {
            return None;
        }

        if residual.norm() < options.residual_tol {
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

        if next_residual.norm() < options.residual_tol && actual_step.norm() < options.step_tol {
            return Some(next);
        }

        current = next;
    }

    None
}
