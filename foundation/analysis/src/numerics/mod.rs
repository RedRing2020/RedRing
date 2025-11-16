//! 数値計算基盤モジュール
//!
//! 特殊数学定数、許容誤差管理、ベクトル・距離計算を提供します。
//! 基本的な数値計算機能は `Scalar` トレイトに統合されています。

pub mod constants;
pub mod vector_distance;
#[cfg(test)]
pub mod vector_distance_tests;

// 特殊数学定数の再エクスポート
pub use constants::{MathConstants, ToleranceConstants};

// ベクトル・距離計算の再エクスポート
pub use vector_distance::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, point_distance_squared, polyline_length, polyline_length_3d, vector_length,
    vector_length_2d, vector_length_3d, vector_length_squared,
};
