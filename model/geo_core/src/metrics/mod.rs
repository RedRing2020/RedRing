//! 幾何学的計量計算モジュール
//!
//! 距離、長さ、面積、体積などの幾何学的な測定値を計算する機能を提供します。

pub mod area_volume;
pub mod distance;
pub mod length;

#[cfg(test)]
mod area_volume_tests;
#[cfg(test)]
mod length_tests;

// 便利な再エクスポート（geometry特化機能 + analysis関数）
// analysisの汎用関数を再エクスポートし、統一インターフェースを提供
pub use area_volume::*; // analysis関数の再エクスポート含む
pub use length::*; // analysis関数の再エクスポート含む

// ellipse機能は approximations/ellipse.rs に統合されました
