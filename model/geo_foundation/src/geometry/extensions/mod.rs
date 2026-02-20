//! Extension Traits - 拡張操作トレイト群
//!
//! 幾何プリミティブの拡張操作を定義するExtensionトレイト群

pub mod boolean_ops; // Boolean演算Extensions
pub mod collision; // 衝突検出Extensions
pub mod intersection; // 交点計算Extensions
pub mod nurbs; // NURBS特有の拡張操作
			   // pub mod transform; // → core/transform.rsに移動
pub mod transform_error; // 変換操作エラー定義

// Re-exports
pub use boolean_ops::*;
pub use collision::*;
pub use intersection::*;
pub use nurbs::*;
// pub use transform::*; // → core::transformでre-export
pub use transform_error::{SafeTransform, TransformError};
