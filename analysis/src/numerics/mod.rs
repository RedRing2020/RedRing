//! 数値計算基盤モジュール
//!
//! 特殊数学定数と許容誤差管理を提供します。
//! 基本的な数値計算機能は `Scalar` トレイトに統合されています。
//!
//! **注意**: このモジュールの定数機能は `crate::consts` に統合されました。
//! 新しいコードでは `crate::consts::{MathConstants, ToleranceConstants}` を使用してください。

// 特殊数学定数の再エクスポート（後方互換性のため）
pub use crate::consts::{MathConstants, ToleranceConstants};
