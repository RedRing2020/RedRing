//! 数値計算基盤モジュール
//!
//! ベクトル・距離計算、数値積分を提供します。
//! ソルバー機能は `crate::linalg::solver` モジュールに移動しました。
//! 特殊数学定数は `crate::consts::special` モジュールを使用してください。

pub mod integration;
pub mod vector_distance;
#[cfg(test)]
pub mod vector_distance_tests;

// 数値積分の再エクスポート
pub use integration::{newton_arc_length, trapezoidal_rule, NormedVector};

// 非線形方程式ソルバーは linalg::solver::newton を使用してください
// pub use crate::linalg::solver::newton::{newton_solve, newton_inverse};

// ベクトル・距離計算の再エクスポート
pub use vector_distance::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, point_distance_squared, polyline_length, polyline_length_3d, vector_length,
    vector_length_2d, vector_length_3d, vector_length_squared,
};
