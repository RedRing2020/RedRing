//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! 他のクレートに依存しない純粋な数値計算機能を提供します。

pub mod abstract_types; // 数値計算の基盤型（Scalar, Angle, Tolerance等）
pub mod consts;
pub mod geometry;
pub mod linalg; // 高速線形代数モジュール（数値解析専用）

// 新しい分類構造
pub mod approximations;
pub mod metrics; // 距離・計量計算
pub mod numerics; // 数値計算基盤 // 幾何学的近似計算

pub mod sampling2d; // 既存の2Dサンプリング（後で整理）

#[cfg(test)]
mod unit_tests;

// 基盤型の再エクスポート
pub use abstract_types::{Angle, AngleType, Scalar, TolerantEq};

// 定数の再エクスポート
pub use consts::{
    game, precision, GeometricTolerance, DEG_TO_RAD, DERIVATIVE_ZERO_THRESHOLD, E,
    GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6,
    RAD_TO_DEG, TAU,
};

// 幾何数値関数の再エクスポート
pub use geometry::{
    basis_function_derivatives, basis_functions, find_span, newton_arc_length, newton_inverse,
    newton_solve, NormedVector,
};
