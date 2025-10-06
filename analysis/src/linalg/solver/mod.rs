/// 連立一次方程式ソルバーモジュール
/// 
/// 直接法と反復法の各種ソルバーを提供
/// - 直接法：ガウス消去法、LU分解法、コレスキー分解法
/// - 反復法：ヤコビ法、ガウス・ザイデル法、SOR法

pub mod gaussian;       // ガウス消去法
pub mod lu;            // LU分解法
pub mod cramer;        // Cramerの公式（既存）

pub use gaussian::GaussianSolver;
pub use lu::LUSolver;
pub use cramer::CramerSolver;

use super::scalar::Scalar;

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