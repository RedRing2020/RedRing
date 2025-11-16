//! 数値計算基盤モジュール
//!
//! ベクトル・距離計算、数値解法ソルバーを提供します。
//! 特殊数学定数は `crate::consts::special` モジュールを使用してください。

pub mod solver;
#[cfg(test)]
pub mod solver_tests;
pub mod vector_distance;
#[cfg(test)]
pub mod vector_distance_tests;

// 数値解法ソルバーの再エクスポート
pub use solver::{newton_arc_length, newton_inverse, newton_solve, NormedVector};

// ベクトル・距離計算の再エクスポート
pub use vector_distance::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, point_distance_squared, polyline_length, polyline_length_3d, vector_length,
    vector_length_2d, vector_length_3d, vector_length_squared,
};
