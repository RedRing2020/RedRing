//! Foundation Traits - 統一操作トレイト群
//!
//! 幾何プリミティブの共通操作を定義するFoundationトレイト

pub mod collision; // 衝突検出Foundation
pub mod intersection; // 交点計算Foundation
pub mod transform; // 変換操作Foundation

// Re-exports
pub use collision::*;
pub use intersection::*;
pub use transform::*;
