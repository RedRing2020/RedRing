//! Extension Traits - 拡張操作トレイト群
//!
//! 幾何プリミティブの拡張操作を定義するExtensionトレイト群

pub mod analysis_conversion; // analysisクレートとの型変換
pub mod collision; // 衝突検出Extensions
pub mod intersection; // 交点計算Extensions
pub mod transform; // 変換操作Extensions
pub mod transform_error; // 変換操作エラー定義

// Re-exports
pub use analysis_conversion::*;
pub use collision::*;
pub use intersection::*;
pub use transform::*;
pub use transform_error::{SafeTransform, TransformError};
