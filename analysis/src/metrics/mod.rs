//! 距離・計量計算モジュール
//!
//! **注意**:
//! - 汎用的な距離・長さ計算: このモジュール
//! - Geometry専用の計算: `geo_core::metrics`

pub mod distance_length;

#[cfg(test)]
mod distance_length_tests;

// 汎用的な距離・長さ計算の再エクスポート
pub use distance_length::*;
