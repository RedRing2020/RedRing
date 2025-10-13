//! Abstract Traits - 最小責務抽象化
//!
//! 幾何プリミティブの最小責務原則に基づく抽象トレイト群
//! 機能は拡張トレイトで分離し、コア機能のみを提供

pub mod arc_traits; // Arc最小責務トレイト
pub mod circle_traits; // Circle最小責務トレイト
pub mod point_traits; // Point最小責務トレイト

// Re-exports
pub use arc_traits::*;
pub use circle_traits::*;
pub use point_traits::*;
