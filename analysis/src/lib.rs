//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! 他のクレートに依存しない純粋な数値計算機能を提供します。

pub mod abstract_types; // 数値計算の基盤型（Scalar, Angle, Tolerance等）
pub mod consts;
pub mod linalg;
pub mod numerical_methods; // 数値計算手法（汎用数値計算 + NURBS/B-spline特化） // 高速線形代数モジュール（数値解析専用）

// 新しい分類構造
pub mod metrics; // 距離・計量計算

pub mod sampling; // 数値サンプリング機能

// テストモジュール（*_tests.rs形式）
#[cfg(test)]
mod interpolation_tests;
#[cfg(test)]
mod numerical_tests;
#[cfg(test)]
mod statistics_tests;

#[cfg(test)]
mod consts_tests;

// 基盤型の再エクスポート
pub use abstract_types::{Angle, AngleType, Scalar, TolerantEq};

// 線形代数の再エクスポート
pub use linalg::matrix::{Matrix3x3, Matrix4x4};
pub use linalg::quaternion::{Quaternion, Quaterniond, Quaternionf};
pub use linalg::vector::{Vector2, Vector3, Vector4};

// 定数の再エクスポート
pub use consts::{
    game, precision, GeometricTolerance, MathConstants, ToleranceConstants, DEG_TO_RAD,
    DERIVATIVE_ZERO_THRESHOLD, E, GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE, PI,
    PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
};

// 数値計算手法の再エクスポート
pub use crate::numerical_methods::{
    basis_function_derivatives, basis_functions, find_span, newton_arc_length, newton_inverse,
    newton_solve, NormedVector,
};

// 汎用距離・長さ計算の再エクスポート
pub use crate::metrics::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, point_distance_squared, polyline_length, polyline_length_3d, vector_length,
    vector_length_2d, vector_length_3d, vector_length_squared,
};
