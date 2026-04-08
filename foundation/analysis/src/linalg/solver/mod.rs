//! 方程式ソルバーモジュール
//!
//! 線形方程式と非線形方程式の各種ソルバーを提供
//! - 線形方程式：ガウス消去法、LU分解法、クラメール法
//! - 非線形方程式：ニュートン・ラフソン法
//!
//! ## 使用例
//! ```rust
//! use analysis::linalg::solver::newton::{newton_inverse_generic, newton_solve_generic};
//!
//! // 非線形方程式 f(x) = x^2 - 2 = 0 の解（√2を求める）
//! let f = |x: f64| x * x - 2.0;
//! let df = |x: f64| 2.0 * x;
//! let result = newton_solve_generic(f, df, 1.0, 100, 1e-10);
//! assert!(result.is_some());
//! ```
pub mod cramer;
pub mod gaussian; // ガウス消去法
pub mod lu; // LU分解法
pub mod newton; // ニュートン・ラフソン法

// Newton法ソルバーの再エクスポート
// generic / multivariate API を正本として公開する。
pub use newton::{
    newton_inverse_generic, newton_solve_bounded_generic, newton_solve_generic,
    newton_solve_multivariate, newton_solve_multivariate_bounded,
    newton_solve_multivariate_bounded_with_solver, newton_solve_multivariate_with_solver,
    newton_solve_with_numeric_derivative_bounded_generic, MultivariateNewtonBounds,
    MultivariateNewtonOptions,
};

// テストモジュール
#[cfg(test)]
pub mod cramer_tests;
#[cfg(test)]
pub mod gaussian_tests;
#[cfg(test)]
pub mod lu_tests;
#[cfg(test)]
pub mod mod_tests;
#[cfg(test)]
pub mod newton_tests;

pub use cramer::CramerSolver;
pub use gaussian::GaussianSolver;
pub use lu::LUSolver;

use crate::abstract_types::Scalar;
use crate::linalg::{DynamicMatrix, Vector};

/// 連立方程式の解法結果
#[derive(Debug, Clone)]
pub struct SolutionInfo<T: Scalar> {
    pub solution: Vec<T>,
    pub residual: T,
    pub iterations: usize,
    pub converged: bool,
}

impl<T: Scalar> SolutionInfo<T> {
    pub fn new(solution: Vec<T>, residual: T, iterations: usize, converged: bool) -> Self {
        Self {
            solution,
            residual,
            iterations,
            converged,
        }
    }

    /// 直接法用のコンストラクタ（反復なし）
    pub fn direct(solution: Vec<T>, residual: T) -> Self {
        Self {
            solution,
            residual,
            iterations: 0,
            converged: true,
        }
    }
}

/// ソルバーの共通トレイト
pub trait LinearSolver<T: Scalar> {
    /// 連立方程式 Ax = b を解く
    fn solve(&self, matrix: &[Vec<T>], rhs: &[T]) -> Result<SolutionInfo<T>, String>;
}

/// DynamicMatrix ベースの入力を既存 solver へ橋渡しする拡張 trait
pub trait DynamicMatrixLinearSolver<T: Scalar>: LinearSolver<T> {
    /// `DynamicMatrix<T>` と `Vector<T>` を受けて既存 solver を実行する
    fn solve_dynamic(
        &self,
        matrix: &DynamicMatrix<T>,
        rhs: &Vector<T>,
    ) -> Result<SolutionInfo<T>, String> {
        if matrix.rows() != matrix.cols() {
            return Err("Matrix must be square".to_string());
        }

        if rhs.len() != matrix.rows() {
            return Err("RHS dimension mismatch".to_string());
        }

        self.solve(&matrix.to_vec2d(), rhs.data())
    }
}

impl<T: Scalar, TLinearSolver> DynamicMatrixLinearSolver<T> for TLinearSolver where
    TLinearSolver: LinearSolver<T>
{
}
