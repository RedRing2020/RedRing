//! 線形代数モジュール
//!
//! 高性能な数値解析向け線形代数ライブラリ
//! - 行列演算（Matrix2x2, Matrix3x3, Matrix4x4）
//! - ベクトル演算（Vector2, Vector3, Vector4）
//! - 点演算（Point2, Point3）
//! - クォータニオン演算（Quaternion）
//! - 連立方程式ソルバー（Gaussian, LU, Cramer）
//! - SIMD最適化対応
//! - 並列処理対応 (将来的にrayon使用予定)

pub mod matrix;
pub mod point2;
pub mod point3;
pub mod quaternion;
pub mod solver;
pub mod vector;

// テストモジュール（*_tests.rs形式）
#[cfg(test)]
pub mod quaternion_tests;
#[cfg(test)]
pub mod scalar_tests;
#[cfg(test)]
pub mod solver_tests;

// 主要型の再エクスポート
pub use matrix::{Matrix2x2, Matrix3x3, Matrix4x4};
pub use quaternion::{Quaternion, Quaterniond, Quaternionf};
pub use solver::{CramerSolver, GaussianSolver, LUSolver, LinearSolver};
pub use vector::{Vector, Vector2, Vector3, Vector4};

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
