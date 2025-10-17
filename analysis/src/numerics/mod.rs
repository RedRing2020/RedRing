//! 数値計算基盤モジュール
//!
//! 特殊数学定数と許容誤差管理を提供します。
//! 基本的な数値計算機能は `Scalar` トレイトに統合されています。

pub mod constants;

// 特殊数学定数の再エクスポート
pub use constants::{MathConstants, ToleranceConstants};
