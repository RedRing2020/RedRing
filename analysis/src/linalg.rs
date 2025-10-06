/// 線形代数演算モジュール
///
/// 高速並列演算を目指した数値解析専用の線形代数ライブラリ
/// - SIMD最適化対応
/// - 並列処理対応 (将来的にrayon使用予定)
/// - ジオメトリ依存を排除した純粋な数値計算

pub mod scalar;
pub mod vector;
pub mod matrix;      // 固定サイズ・動的サイズ行列
pub mod quaternion;  // クォータニオン（四元数）
pub mod solver;

#[cfg(test)]
pub mod unit_tests;

// 明示的なインポートで競合を解決
pub use scalar::*;
pub use vector::*;
pub use matrix::{Matrix2x2, Matrix3x3, Matrix4x4};
pub use quaternion::{Quaternion, Quaternionf, Quaterniond};
pub use solver::*;

// 便利な型エイリアス（ベクトル）
pub type Vec2f = Vector2<f32>;
pub type Vec3f = Vector3<f32>;
pub type Vec4f = Vector4<f32>;

pub type Vec2d = Vector2<f64>;
pub type Vec3d = Vector3<f64>;
pub type Vec4d = Vector4<f64>;

// 行列の型エイリアス
pub type Mat2f = Matrix2x2<f32>;
pub type Mat3f = Matrix3x3<f32>;
pub type Mat4f = Matrix4x4<f32>;

pub type Mat2d = Matrix2x2<f64>;
pub type Mat3d = Matrix3x3<f64>;
pub type Mat4d = Matrix4x4<f64>;
