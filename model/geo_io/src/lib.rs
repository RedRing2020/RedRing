//! geo_io - 幾何データのファイルI/Oクレート
//!
//! STL、OBJ、PLY等の3Dメッシュファイル形式の読み書きを提供します。
//!
//! # サポートフォーマット
//! - STL (ASCII/Binary)
//! - OBJ (予定)
//! - PLY (予定)
//!
//! # 使用例
//! 詳細な使用例は `manual/geo_io_examples.md` を参照してください。

pub mod error;
pub mod fixtures;
pub mod stl;
pub mod stl_bulk;
pub mod svg;

// Re-exports
pub use error::{IoError, StlError};
pub use stl_bulk::{StlIndexedBulk, StlTriangleBulk};
