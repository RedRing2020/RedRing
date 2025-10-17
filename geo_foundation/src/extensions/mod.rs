//! Extension Traits - 拡張操作トレイト群
//!
//! 幾何プリミティブの拡張操作を定義するExtensionトレイト群

pub mod collision; // 衝突検出Extensions
pub mod intersection; // 交点計算Extensions
pub mod transform; // 変換操作Extensions

// Re-exports
pub use collision::*;
pub use intersection::*;
pub use transform::*;
