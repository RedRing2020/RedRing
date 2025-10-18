//! 行列演算モジュール
//!
//! 固定サイズと動的サイズの行列演算を提供
//! - 2x2, 3x3, 4x4行列（固定サイズ、高速演算）
//! - 動的サイズ行列（汎用性）
//! - 将来的に疎行列、特殊行列も対応予定
pub mod matrix2; // 2x2行列
pub mod matrix3; // 3x3行列
pub mod matrix4; // 4x4行列

// テストモジュール（*_tests.rs形式）
#[cfg(test)]
pub mod matrix2_tests;
#[cfg(test)]
pub mod matrix3_tests;
#[cfg(test)]
pub mod matrix4_tests;

pub use matrix2::{Matrix2x2, Matrix2x2d, Matrix2x2f};
pub use matrix3::{Matrix3x3, Matrix3x3d, Matrix3x3f};
pub use matrix4::{Matrix4x4, Matrix4x4d, Matrix4x4f};
