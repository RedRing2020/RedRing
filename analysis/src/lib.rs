//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! 他のクレートに依存しない純粋な数値計算機能を提供します。

pub mod abstract_types; // 数値計算の基盤型（Scalar, Angle, Tolerance等）
pub mod consts;
pub mod geometry;
pub mod linalg; // 高速線形代数モジュール（数値解析専用）
pub mod sampling2d; // 既存の2Dサンプリング（後で整理）

#[cfg(test)]
mod unit_tests;

// 基盤型の再エクスポート
pub use abstract_types::{
    Angle, AngleType, Scalar, ToleranceContext, TolerantEq, GEOMETRIC_TOLERANCE,
};

// 定数の再エクスポート
pub use consts::DERIVATIVE_ZERO_THRESHOLD;

// 幾何数値関数の再エクスポート
pub use geometry::{
    basis_function_derivatives, basis_functions, find_span, newton_arc_length, newton_inverse,
    newton_solve, NormedVector,
};
