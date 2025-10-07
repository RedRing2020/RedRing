//! 抽象型定義モジュール
//!
//! 幾何計算で使用される基本的な抽象型を定義

pub mod geometry;
mod scalar;
mod tolerance;

pub use scalar::Scalar;
pub use tolerance::{ToleranceContext, TolerantEq};
