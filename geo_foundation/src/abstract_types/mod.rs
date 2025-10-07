//! 抽象型定義モジュール
//!
//! 幾何計算で使用される基本的な抽象型を定義

mod scalar;
mod tolerance;
pub mod geometry;

pub use scalar::Scalar;
pub use tolerance::{ToleranceContext, TolerantEq};

// 旧統合バージョン（移行期間中のみ保持）
mod geometry_unified;
pub use geometry_unified::*;
