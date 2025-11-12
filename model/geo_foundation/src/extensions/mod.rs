//! Extension Traits - 拡張操作トレイト群
//!
//! 幾何プリミティブの拡張操作を定義するExtensionトレイト群

pub mod analysis_conversion; // analysisクレートとの型変換 - 抽象トレイト定義
pub mod analysis_transform; // Analysis Matrix/Vector変換 - 効率的変換トレイト群
pub mod collision; // 衝突検出Extensions
pub mod intersection; // 交点計算Extensions
pub mod nurbs; // NURBS特有の拡張操作
pub mod transform; // 変換操作Extensions - 抽象トレイト定義
pub mod transform_error; // 変換操作エラー定義

// Re-exports
pub use analysis_conversion::*; // 抽象トレイト定義
pub use analysis_transform::*; // Analysis変換トレイト群
pub use collision::*;
pub use intersection::*;
pub use nurbs::*;
pub use transform::*; // 抽象トレイト定義
pub use transform_error::{SafeTransform, TransformError};
