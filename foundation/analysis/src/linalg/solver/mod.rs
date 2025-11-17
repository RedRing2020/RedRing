//! 方程式ソルバーモジュール
//!
//! 線形方程式と非線形方程式の各種ソルバーを提供
//! - 線形方程式：ガウス消去法、LU分解法、クラメール法
//! - 非線形方程式：ニュートン・ラフソン法
//!
//! ## 使用例
//! ```rust
//! use analysis::linalg::solver::newton::{newton_solve, newton_inverse};
//! 
//! // 非線形方程式 f(x) = x^2 - 2 = 0 の解（√2を求める）
//! let f = |x: f64| x * x - 2.0;
//! let df = |x: f64| 2.0 * x;
//! let result = newton_solve(f, df, 1.0, 100, 1e-10);
//! assert!(result.is_some());
//! ```
pub mod cramer;
pub mod gaussian; // ガウス消去法
pub mod lu; // LU分解法
pub mod newton; // ニュートン・ラフソン法

// テストモジュール
#[cfg(test)]
pub mod solver_tests;

pub use cramer::CramerSolver;
pub use gaussian::GaussianSolver;
pub use lu::LUSolver;

use crate::abstract_types::Scalar;

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
